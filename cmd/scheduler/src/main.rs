use std::{num::NonZeroU64, sync::Arc, time::Duration};

use stores::{
    config::{ConfigStore, PremiumSlotTier},
    postgres::Postgres,
};
use tokio::sync::mpsc;
use tracing::{error, info};
use twilight_model::id::Id;

use crate::vmworkerpool::WorkerLaunchConfig;

mod broker_client;
mod command_manager;
mod dispatch_conv;
mod guild_handler;
mod integration_testing;
mod interval_timer_manager;
mod rpc_server;
mod scheduled_task_manager;
mod scheduler;
mod vm_session;
mod vmworkerpool;
mod worker_listener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: SchedulerConfig = common::load_config();
    common::setup_tracing(&config.common, "scheduler");
    common::setup_metrics("0.0.0.0:7803");

    let discord_config = common::fetch_discord_config(config.common.discord_token.clone())
        .await
        .expect("failed fetching discord config");
    let integration_testing_guild = config.integration_tests_guild.map(Id::from);

    info!("launching scheduler!");
    let postgres_store = Arc::new(
        Postgres::new_with_url(&config.common.database_url)
            .await
            .unwrap(),
    );

    let guild_log_sub_backend =
        Arc::new(guild_logger::guild_subscriber_backend::GuildSubscriberBackend::default());

    let (logger, ig_testing_tracker) = {
        let mut builder = guild_logger::GuildLoggerBuilder::new()
            .add_backend(Arc::new(guild_logger::discord_backend::DiscordLogger::new(
                discord_config.clone(),
                postgres_store.clone(),
            )))
            .add_backend(guild_log_sub_backend.clone());

        if let Some(g) = integration_testing_guild {
            // get number of ig testing scripts
            let scripts = postgres_store.list_scripts(g).await?;
            let tracker = Arc::new(integration_testing::Tracker::new(scripts.len() as i32));
            builder = builder.add_backend(tracker.clone());
            (builder.run(), Some(tracker))
        } else {
            (builder.run(), None)
        }
    };

    let (scheduler_tx, scheduler_rx) = mpsc::unbounded_channel();

    let bot_rpc_server = rpc_server::Server::new(
        guild_log_sub_backend,
        scheduler_tx.clone(),
        config.common.bot_rpc_listen_addr.clone(),
    );
    tokio::spawn(bot_rpc_server.run());

    let (manager, cmd_man_handle) = command_manager::create_manager_pair(
        postgres_store.clone(),
        discord_config.clone(),
        logger.clone(),
    );

    tokio::spawn(manager.run());

    let worker_pool = vmworkerpool::VmWorkerPool::new(WorkerLaunchConfig {
        cmd: config.vmworker_bin_path.clone(),
    });

    #[cfg(target_family = "unix")]
    worker_listener::listen_for_workers("/tmp/botloader_scheduler_workers", worker_pool.clone())
        .await;

    #[cfg(target_family = "windows")]
    worker_listener::listen_for_workers("localhost:7885", worker_pool.clone()).await;

    tokio::time::sleep(Duration::from_secs(1)).await;
    info!(
        "spawning {},{},{}, free, lite, premium workers",
        config.num_workers_free, config.num_workers_lite, config.num_workers_premium,
    );
    worker_pool.spawn_workers(None, config.num_workers_free as usize);
    worker_pool.spawn_workers(
        Some(PremiumSlotTier::Lite),
        config.num_workers_lite as usize,
    );
    worker_pool.spawn_workers(
        Some(PremiumSlotTier::Premium),
        config.num_workers_premium as usize,
    );

    let scheduler = scheduler::Scheduler::new(
        Arc::new(config.clone()),
        scheduler_rx,
        postgres_store,
        logger,
        cmd_man_handle,
        worker_pool,
    );
    let task = tokio::spawn(scheduler.run());

    tokio::spawn(broker_client::broker_client(
        config.broker_rpc_connect_adddr,
        scheduler_tx.clone(),
    ));

    if integration_testing_guild.is_none() {
        common::shutdown::wait_shutdown_signal().await;
    } else {
        // termination is handled by the vm
        let tracker = ig_testing_tracker.unwrap();
        loop {
            match tracker.is_complete() {
                integration_testing::CompletionStatus::Pending => {}
                integration_testing::CompletionStatus::Timeout => {
                    error!("testing timed out");
                    std::process::exit(1);
                }
                integration_testing::CompletionStatus::Complete(num) => {
                    info!("testing was completed successfully! ran {} scripts.", num);
                    break;
                }
                integration_testing::CompletionStatus::Error(err) => {
                    error!(%err, "testing resulted in a error");
                    std::process::exit(1);
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    let _ = scheduler_tx.send(scheduler::SchedulerCommand::Shutdown);

    info!("shutting down....");

    let _ = task.await;

    Ok(())
}

#[derive(Clone, clap::Parser)]
pub struct SchedulerConfig {
    #[clap(flatten)]
    pub(crate) common: common::config::RunConfig,

    #[clap(
        long,
        env = "BL_BROKER_RPC_CONNECT_ADDR",
        default_value = "0.0.0.0:7480"
    )]
    pub(crate) broker_rpc_connect_adddr: String,

    #[clap(long)]
    pub integration_tests_guild: Option<NonZeroU64>,

    #[clap(
        long,
        env = "BL_VMWORKER_BIN_PATH",
        default_value = "../../target/debug/vmworker"
    )]
    pub vmworker_bin_path: String,

    #[clap(long, env = "BL_SCHEDULER_NUM_WORKERS_FREE", default_value = "2")]
    pub(crate) num_workers_free: u16,
    #[clap(long, env = "BL_SCHEDULER_NUM_WORKERS_LITE", default_value = "0")]
    pub(crate) num_workers_lite: u16,
    #[clap(long, env = "BL_SCHEDULER_NUM_WORKERS_PREMIUM", default_value = "0")]
    pub(crate) num_workers_premium: u16,

    // Disables reusing vm's when the vm session has to grab a worker from the pool
    // This is useful for benchmarking and diagnostics purposes
    #[clap(long, env = "BL_SCHEDULER_NO_REUSE_VMS", default_value = "false")]
    pub no_reuse_vms: bool,
}
