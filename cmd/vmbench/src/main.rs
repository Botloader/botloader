use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use clap::{Parser, Subcommand};
use common::discord::DiscordConfig;
use runtime::CreateRuntimeContext;
use stores::config::{Script, ScriptContributes};
use tokio::sync::mpsc;
use twilight_model::{
    id::{marker::GuildMarker, Id},
    oauth::Application,
    user::CurrentUser,
};
use vm::vm::{CreateRt, ShutdownReason, VmEvent};

#[derive(Parser)]
#[command(about = "benchmark botloader vm creations")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Benchmarks vm creation in-process using the same code path as the vm
    /// worker. Needs no infrastructure: the runtime context is stubbed out
    /// and only touched if a script calls storage/discord APIs.
    ///
    /// Measures from just before the vm thread is spawned until the vm
    /// reports VmFinished (isolate created from the snapshot, all scripts
    /// compiled and ran).
    Vm(VmArgs),

    /// Benchmarks the full scheduler flow with real vm worker child
    /// processes: event in -> guild activation (premium tier + scripts from
    /// db) -> worker claimed -> create vm rpc -> vm created -> event
    /// dispatched -> acked.
    ///
    /// Needs the same environment as the integration tests: DATABASE_URL
    /// (scripts are seeded into the db) and DISCORD_BOT_TOKEN /
    /// DISCORD_CLIENT_ID / DISCORD_CLIENT_SECRET for the workers. Don't run
    /// this while a real scheduler is running on the same machine, they share
    /// the worker socket.
    Full(FullArgs),

    /// Internal: runs a vm worker process, spawned by the `full` subcommand.
    #[command(hide = true)]
    VmWorker(vmworker::WorkerConfig),
}

#[derive(clap::Args)]
struct VmArgs {
    /// Paths to .ts scripts to load into each vm (may use the botloader API,
    /// but must not call storage/discord APIs)
    scripts: Vec<PathBuf>,

    /// Number of measured vm creations
    #[arg(long, default_value_t = 25)]
    iterations: usize,

    /// Number of unmeasured warmup vm creations
    #[arg(long, default_value_t = 3)]
    warmup: usize,

    /// Don't print guild logs (script errors, console.log output) to stderr
    #[arg(long)]
    quiet: bool,
}

#[derive(clap::Args)]
struct FullArgs {
    /// Paths to .ts scripts, they replace the bench guild's scripts in the
    /// database (at least one is required, events aren't dispatched to guilds
    /// without scripts)
    scripts: Vec<PathBuf>,

    /// Number of measured runs
    #[arg(long, default_value_t = 10)]
    iterations: usize,

    /// Number of unmeasured warmup runs
    #[arg(long, default_value_t = 2)]
    warmup: usize,

    /// Number of vm worker processes to spawn (the bench itself only uses one
    /// at a time)
    #[arg(long, default_value_t = 1)]
    workers: usize,

    /// Guild id to bench with. WARNING: this guild's scripts in the database
    /// are replaced with the provided ones
    #[arg(
        long,
        env = "VMBENCH_GUILD_ID",
        default_value_t = 999_000_000_000_000_001
    )]
    guild_id: u64,

    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// Don't print guild logs (script errors, console.log output) to stderr
    #[arg(long)]
    quiet: bool,
}

const GUILD_ID: Id<GuildMarker> = Id::new(1);

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Command::VmWorker(config) = args.command {
        // workers set up their own tracing
        vmworker::run(config).await.unwrap();
        return;
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "error".into()),
        )
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    match args.command {
        Command::Vm(vm_args) => run_vm_bench(vm_args).await,
        Command::Full(full_args) => run_full_bench(full_args).await,
        Command::VmWorker(_) => unreachable!(),
    }
}

async fn run_vm_bench(args: VmArgs) {
    let scripts = load_scripts(&args.scripts);
    let shared = SharedContext::new(args.quiet);

    println!(
        "benchmarking in-process vm creation: {} scripts per vm, {} warmup + {} measured iterations",
        scripts.len(),
        args.warmup,
        args.iterations
    );

    for _ in 0..args.warmup {
        create_one(&shared, scripts.clone()).await;
    }

    let mut samples = Vec::with_capacity(args.iterations);
    for i in 0..args.iterations {
        let timings = create_one(&shared, scripts.clone()).await;
        println!(
            "iteration {:>3}: created vm in {:>8.2?} (of which thread spawn + handles {:.2?})",
            i + 1,
            timings.finished,
            timings.spawned
        );
        samples.push(timings);
    }

    println!(
        "\nall times are per vm creation, over {} measured creations:",
        samples.len()
    );
    print_stats(
        "vm creation (spawn -> VmFinished)",
        samples.iter().map(|v| v.finished),
    );
    print_stats("thread spawn + handles", samples.iter().map(|v| v.spawned));
}

async fn run_full_bench(args: FullArgs) {
    for var in [
        "DISCORD_BOT_TOKEN",
        "DISCORD_CLIENT_ID",
        "DISCORD_CLIENT_SECRET",
    ] {
        if std::env::var(var).is_err() {
            eprintln!("{var} must be set, the vm worker processes need it");
            std::process::exit(1);
        }
    }

    let scripts = load_scripts(&args.scripts);
    if scripts.is_empty() {
        eprintln!("the full flow bench needs at least one script, events aren't dispatched to guilds without scripts");
        std::process::exit(1);
    }

    let db = stores::Db::new_with_url(&args.database_url)
        .await
        .expect("connect to database");

    let mut logger_builder = guild_logger::GuildLoggerBuilder::new();
    if !args.quiet {
        logger_builder = logger_builder.add_backend(Arc::new(StderrLogBackend));
    }
    let logger = logger_builder.run();

    let current_exe = std::env::current_exe().expect("current exe path");
    let launch_config = scheduler::WorkerLaunchConfig {
        cmd: current_exe.to_str().expect("utf-8 exe path").to_string(),
        args: vec!["vm-worker".to_string()],
    };

    println!(
        "benchmarking full flow: {} scripts, {} workers, {} warmup + {} measured iterations",
        scripts.len(),
        args.workers,
        args.warmup,
        args.iterations
    );
    println!("spawning workers and waiting for them to connect...");

    let mut bencher = scheduler::bench::FullFlowBencher::new(
        scheduler::bench::FullFlowBenchOptions {
            db,
            discord_config: Arc::new(stub_discord_config()),
            logger,
            launch_config,
            num_workers: args.workers,
            guild_id: Id::new(args.guild_id),
        },
        scripts
            .into_iter()
            .map(|s| scheduler::bench::BenchScript {
                name: s.name,
                source: s.original_source,
            })
            .collect(),
    )
    .await
    .expect("full flow bench setup");

    for _ in 0..args.warmup {
        bencher.run_iteration().await.expect("warmup iteration");
    }

    let mut samples = Vec::with_capacity(args.iterations);
    for i in 0..args.iterations {
        let timings = bencher.run_iteration().await.expect("bench iteration");
        println!(
            "iteration {:>3}: event acked in {:>8.2?} (vm requested after {:.2?})",
            i + 1,
            timings.event_acked,
            timings.vm_requested
        );
        samples.push(timings);
    }

    println!(
        "\nall times are per run (event in -> guild activation -> vm created -> event acked), over {} measured runs:",
        samples.len()
    );
    print_stats(
        "event in -> event acked by vm",
        samples.iter().map(|v| v.event_acked),
    );
    print_stats(
        "event in -> create vm request sent (db queries + worker claim)",
        samples.iter().map(|v| v.vm_requested),
    );
}

/// Context shared between iterations, mirroring what the vm worker keeps
/// alive across vm creations.
struct SharedContext {
    guild_logger: guild_logger::GuildLogSender,
    discord_config: Arc<DiscordConfig>,
    broker_client: dbrokerapi::state_client::Client,
    db: stores::Db,
    premium_tier: Arc<RwLock<Option<stores::config::PremiumSlotTier>>>,
}

impl SharedContext {
    fn new(quiet: bool) -> Self {
        let mut logger_builder = guild_logger::GuildLoggerBuilder::new();
        if !quiet {
            logger_builder = logger_builder.add_backend(Arc::new(StderrLogBackend));
        }

        Self {
            guild_logger: logger_builder.run().with_guild(GUILD_ID),
            discord_config: Arc::new(stub_discord_config()),
            broker_client: dbrokerapi::state_client::Client::new("http://127.0.0.1:1".to_string()),
            db: stores::Db::new_with_url_lazy("postgres://localhost/vmbench-unused")
                .expect("create lazy db pool"),
            premium_tier: Arc::new(RwLock::new(None)),
        }
    }
}

struct Timings {
    /// Time until spawn_vm_thread returned (thread spawned, shutdown handle received)
    spawned: Duration,
    /// Time until the vm reported VmFinished (isolate created, scripts compiled and ran)
    finished: Duration,
}

async fn create_one(shared: &SharedContext, scripts: Vec<Script>) -> Timings {
    let (vm_cmd_tx, vm_cmd_rx) = mpsc::unbounded_channel();
    let (vm_evt_tx, mut vm_evt_rx) = mpsc::unbounded_channel();
    let (runtime_evt_tx, _runtime_evt_rx) = mpsc::unbounded_channel();

    let rt_ctx = CreateRuntimeContext {
        bot_state: shared.broker_client.clone(),
        discord_config: shared.discord_config.clone(),
        guild_id: Some(GUILD_ID),
        guild_logger: shared.guild_logger.clone(),
        script_http_client_proxy: None,
        premium_tier: shared.premium_tier.clone(),
        main_tokio_runtime: tokio::runtime::Handle::current(),
        settings_values: Vec::new(),
        db: shared.db.clone(),
        event_tx: runtime_evt_tx,
    };

    let started = Instant::now();

    let shutdown_handle = vm::vmthread::spawn_vm_thread(
        CreateRt {
            guild_logger: shared.guild_logger.clone(),
            rx: vm_cmd_rx,
            tx: vm_evt_tx,
            load_scripts: scripts,
            extension_factory: Box::new(move || runtime::create_extensions(rt_ctx.clone())),
            extension_modules: runtime::jsmodules::create_module_map(),
        },
        || tracing::info_span!("vmthread"),
    )
    .await;

    let spawned = started.elapsed();

    let finished = loop {
        match vm_evt_rx.recv().await {
            Some(VmEvent::VmFinished) => break started.elapsed(),
            Some(_) => continue,
            None => panic!("vm event channel closed before the vm finished starting"),
        }
    };

    shutdown_handle.shutdown_vm(ShutdownReason::Request, false);
    // the vm thread drops the command receiver when it shuts down
    vm_cmd_tx.closed().await;

    Timings { spawned, finished }
}

fn load_scripts(paths: &[PathBuf]) -> Vec<Script> {
    paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let source = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("failed reading {}: {}", path.display(), e));
            let name = path
                .file_stem()
                .expect("script path has a file name")
                .to_string_lossy()
                .to_string();

            Script {
                id: i as u64 + 1,
                name,
                original_source: source,
                enabled: true,
                contributes: ScriptContributes {
                    commands: Vec::new(),
                    interval_timers: Vec::new(),
                },
                plugin_id: None,
                plugin_auto_update: None,
                plugin_version_number: None,
                settings_definitions: None,
                settings_values: Vec::new(),
            }
        })
        .collect()
}

fn print_stats(name: &str, samples: impl Iterator<Item = Duration>) {
    let mut sorted: Vec<Duration> = samples.collect();
    sorted.sort();

    let total: Duration = sorted.iter().sum();
    let mean = total / sorted.len() as u32;
    let percentile = |p: f64| sorted[((sorted.len() - 1) as f64 * p).round() as usize];

    println!("\n{name}:");
    println!(
        "  min {:.2?} | mean {:.2?} | p50 {:.2?} | p90 {:.2?} | max {:.2?}",
        sorted[0],
        mean,
        percentile(0.5),
        percentile(0.9),
        sorted[sorted.len() - 1],
    );
}

/// Prints guild logs (script compile errors, console.log output) to stderr
/// so script failures don't go unnoticed.
struct StderrLogBackend;

#[async_trait::async_trait]
impl guild_logger::GuildLoggerBackend for StderrLogBackend {
    async fn handle_entry(&self, entry: guild_logger::LogEntry) {
        eprintln!("[script log] {}: {}", entry.level, entry.message);
    }
}

/// The runtime context requires a discord config, but nothing in it is used
/// during vm creation, so stub values are fine.
fn stub_discord_config() -> DiscordConfig {
    let bot_user: CurrentUser = serde_json::from_value(serde_json::json!({
        "id": "1",
        "username": "vmbench",
        "discriminator": "0000",
        "avatar": null,
        "bot": true,
        "mfa_enabled": false,
        "verified": true,
    }))
    .expect("valid stub CurrentUser");

    let application: Application = serde_json::from_value(serde_json::json!({
        "id": "1",
        "name": "vmbench",
        "description": "",
        "icon": null,
        "bot_public": false,
        "bot_require_code_grant": false,
        "verify_key": "",
    }))
    .expect("valid stub Application");

    DiscordConfig {
        bot_user,
        application,
        owners: Vec::new(),
        client: twilight_http::Client::new("vmbench-fake-token".to_string()),
    }
}
