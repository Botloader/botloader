use std::sync::{Arc, RwLock};

use common::DiscordConfig;
use guild_logger::LogSender;
use runtime::{CreateRuntimeContext, RuntimeEvent, ScriptSettingsValues};
use scheduler_worker_rpc::{
    CreateScriptsVmReq, SchedulerMessage, VmSessionShutdownEvent, WorkerMessage,
};
use stores::{config::PremiumSlotTier, Db};
use tokio::sync::mpsc;
use tracing::{error, info, instrument};
use twilight_model::id::{marker::GuildMarker, Id};
use vm::vm::{CreateRt, VmCommand, VmEvent, VmShutdownHandle};

mod metrics_forwarder;

pub async fn run(config: WorkerConfig) -> Result<(), Box<dyn std::error::Error>> {
    common::setup_tracing(&config.common, "vmworker");

    let discord_config = common::fetch_discord_config(config.common.discord_token.clone())
        .await
        .expect("failed fetching discord config");

    info!("worker starting");

    #[cfg(target_family = "unix")]
    let (scheduler_tx, scheduler_rx) =
        connect_scheduler("/tmp/botloader_scheduler_workers", config.worker_id).await;

    #[cfg(target_family = "windows")]
    let (scheduler_tx, scheduler_rx) = connect_scheduler("localhost:7885", config.worker_id).await;

    metrics::set_global_recorder(metrics_forwarder::MetricsForwarder {
        tx: scheduler_tx.clone(),
    })
    .expect("set metrics recorder");

    let postgres_store = Db::new_with_url(&config.common.database_url).await.unwrap();

    // suppress signals for now
    // TODO: remove this? do we need signals here?
    // ideally we wanna manage this through the parent
    tokio::spawn(common::shutdown::wait_shutdown_signal());

    let logger = {
        let builder =
            guild_logger::GuildLoggerBuilder::new().add_backend(Arc::new(GuildLogForwarder {
                tx: scheduler_tx.clone(),
            }));

        builder.run()
    };

    let broker_client = dbrokerapi::state_client::Client::new(config.broker_api_addr);

    vm::init_v8_platform();

    let worker = Worker::new(
        scheduler_rx,
        scheduler_tx,
        postgres_store,
        logger,
        discord_config,
        config.common.user_script_http_proxy.clone(),
        broker_client,
    );

    worker.run().await;
    info!("worker shutting down");

    Ok(())
}

#[derive(Clone, clap::Parser)]
pub struct WorkerConfig {
    #[clap(flatten)]
    pub(crate) common: common::config::RunConfig,

    #[clap(
        long,
        env = "BL_BROKER_API_ADDR",
        default_value = "http://0.0.0.0:7449"
    )]
    pub(crate) broker_api_addr: String,

    #[clap(long, env = "BL_WORKER_ID")]
    pub(crate) worker_id: u64,
}

struct WorkerState {
    guild_id: Id<GuildMarker>,
    shutdown_handle: VmShutdownHandle,
    scripts_vm: mpsc::UnboundedSender<VmCommand>,
    evt_rx: mpsc::UnboundedReceiver<VmEvent>,
    session_id: u64,
}

struct Worker {
    scheduler_rx: mpsc::UnboundedReceiver<SchedulerMessage>,
    scheduler_tx: mpsc::UnboundedSender<WorkerMessage>,
    runtime_evt_rx: mpsc::UnboundedReceiver<RuntimeEvent>,
    runtime_evt_tx: mpsc::UnboundedSender<RuntimeEvent>,

    guild_logger: guild_logger::LogSender,
    discord_config: Arc<DiscordConfig>,
    user_http_proxy: Option<String>,
    broker_client: dbrokerapi::state_client::Client,

    premium_tier: Arc<RwLock<Option<PremiumSlotTier>>>,
    stores: Db,
    current_state: Option<WorkerState>,
}

impl Worker {
    fn new(
        scheduler_rx: mpsc::UnboundedReceiver<SchedulerMessage>,
        scheduler_tx: mpsc::UnboundedSender<WorkerMessage>,
        stores: Db,
        guild_logger: LogSender,
        discord_config: Arc<DiscordConfig>,
        user_http_proxy: Option<String>,
        broker_client: dbrokerapi::state_client::Client,
    ) -> Self {
        let (runtime_evt_tx, runtime_evt_rx) = mpsc::unbounded_channel();

        Self {
            scheduler_rx,
            scheduler_tx,
            runtime_evt_rx,
            runtime_evt_tx,
            stores,
            guild_logger,
            discord_config,
            user_http_proxy,
            broker_client,
            current_state: None,
            premium_tier: Arc::new(RwLock::new(None)),
        }
    }

    async fn run(mut self) {
        loop {
            let res = if let Some(current) = &mut self.current_state {
                tokio::select! {
                    scheduler_cmd = self.scheduler_rx.recv() => {
                        if let Some(cmd) = scheduler_cmd{
                            self.handle_scheduler_cmd(cmd).await
                        }else{
                            Ok(ContinueState::Stop)
                        }
                    }
                    runtime_event = self.runtime_evt_rx.recv() => {
                        if let Some(evt) = runtime_event{
                            self.handle_runtime_evt(evt).await
                        }else{
                            Ok(ContinueState::Stop)
                        }
                    }
                    vm_event = current.evt_rx.recv() => {
                        if let Some(evt) = vm_event{
                            self.handle_vm_evt(evt).await
                        }else{
                            info!("vm shut down: channel closed");
                            let current = self.current_state.take().unwrap();
                            self.handle_vm_channel_closed(&current).await.map(|_|ContinueState::Continue)
                        }
                    }
                }
            } else {
                tokio::select! {
                    scheduler_cmd = self.scheduler_rx.recv() => {
                        if let Some(cmd) = scheduler_cmd{
                            self.handle_scheduler_cmd(cmd).await
                        }else{
                            Ok(ContinueState::Stop)
                        }
                    }
                    runtime_event = self.runtime_evt_rx.recv() => {
                        if let Some(evt) = runtime_event{
                            self.handle_runtime_evt(evt).await
                        }else{
                            Ok(ContinueState::Stop)
                        }
                    }
                }
            };

            match res {
                Err(err) => {
                    error!(%err, "failed sending scheduler message")
                }
                Ok(ContinueState::Stop) => break,
                Ok(ContinueState::Continue) => {}
            }
        }

        if let Err(err) = self.wait_shutdown_current_vm().await {
            error!(%err, "failed shutting down current vm")
        }
    }

    #[instrument(
        skip(self, cmd),
        fields(
            guild_id = ?self.span_guild_id_or_scheduler_message_guild_id(&cmd),
            message_type = cmd.span_name()
        )
    )]
    async fn handle_scheduler_cmd(
        &mut self,
        cmd: SchedulerMessage,
    ) -> anyhow::Result<ContinueState> {
        match cmd {
            SchedulerMessage::Dispatch(evt) => {
                info!("worker is dispatching {}", evt.name);
                if let Some(current) = &self.current_state {
                    let _ = current.scripts_vm.send(VmCommand::DispatchEvent(evt));
                }

                Ok(ContinueState::Continue)
            }
            SchedulerMessage::Shutdown => Ok(ContinueState::Stop),
            SchedulerMessage::CreateScriptsVm(data) => self.handle_create_scripts_vm(data).await,
            SchedulerMessage::Complete => {
                // complete the vm
                if let Some(current) = &self.current_state {
                    current
                        .shutdown_handle
                        .shutdown_vm(vm::vm::ShutdownReason::Request, false);
                }
                Ok(ContinueState::Continue)
            }
        }
    }

    #[instrument(
        skip(self, evt),
        fields(
            guild_id = ?self.current_guild_id(),
            evt = evt.span_name()
        )
    )]
    async fn handle_runtime_evt(&mut self, evt: RuntimeEvent) -> anyhow::Result<ContinueState> {
        match evt {
            RuntimeEvent::ScriptStarted(sm) => {
                self.write_message(WorkerMessage::ScriptStarted(sm)).await?;
            }
            RuntimeEvent::NewTaskScheduled => {
                self.write_message(WorkerMessage::TaskScheduled).await?;
            }
        }
        Ok(ContinueState::Continue)
    }

    #[instrument(
        skip(self),
        fields(
            guild_id = ?self.current_guild_id(),
        )
    )]
    async fn handle_vm_evt(&mut self, evt: VmEvent) -> anyhow::Result<ContinueState> {
        match evt {
            VmEvent::DispatchedEvent(id) => self.write_message(WorkerMessage::Ack(id)).await?,
            VmEvent::VmFinished => {
                while let Ok(evt) = self.runtime_evt_rx.try_recv() {
                    self.handle_runtime_evt(evt).await?;
                }
                self.guild_logger.flush().await;
                self.write_message(WorkerMessage::NonePending).await?;
                info!("vm finished");
            }
        }
        Ok(ContinueState::Continue)
    }

    #[instrument(skip_all)]
    async fn handle_create_scripts_vm(
        &mut self,
        req: CreateScriptsVmReq,
    ) -> anyhow::Result<ContinueState> {
        if self.current_state.is_some() {
            self.wait_shutdown_current_vm().await?;
        };

        {
            // update premium tier
            let mut w = self.premium_tier.write().unwrap();
            *w = req.premium_tier;
        }

        // if let Some(current) = &self.current_state {
        //     // we were already running a vm for this guild, issue a restart command with the new scripts instead
        //     // TODO: there is a possibility of a race condition here
        //     // we could receive a "completed" event here we handle after this and since we send a ack back
        //     // stuff could go wrong...
        //     let _ = current.scripts_vm.send(VmCommand::Restart(req.scripts));
        //     self.write_message(WorkerMessage::Ack(req.seq)).await?;
        //     return Ok(ContinueState::Continue);
        // }

        let (vm_cmd_tx, vm_cmd_rx) = mpsc::unbounded_channel();
        let (vm_evt_tx, vm_evt_rx) = mpsc::unbounded_channel();

        let rt_ctx = CreateRuntimeContext {
            bot_state: self.broker_client.clone(),
            discord_config: self.discord_config.clone(),
            guild_id: Some(req.guild_id),
            guild_logger: self.guild_logger.with_guild(req.guild_id),
            script_http_client_proxy: self.user_http_proxy.clone(),
            premium_tier: self.premium_tier.clone(),
            main_tokio_runtime: tokio::runtime::Handle::current(),

            settings_values: req
                .scripts
                .iter()
                .filter(|v| !v.settings_values.is_empty())
                .map(|v| ScriptSettingsValues {
                    script_id: v.id,
                    settings_values: v.settings_values.clone(),
                })
                .collect(),

            db: self.stores.clone(),
            event_tx: self.runtime_evt_tx.clone(),
        };

        let vmthread = vm::vmthread::spawn_vm_thread(
            CreateRt {
                guild_logger: self.guild_logger.with_guild(req.guild_id),
                rx: vm_cmd_rx,
                tx: vm_evt_tx,
                load_scripts: req.scripts,

                extension_factory: Box::new(move || runtime::create_extensions(rt_ctx.clone())),
                extension_modules: runtime::jsmodules::create_module_map(),
            },
            move || tracing::info_span!("vmthread", guild_id = %req.guild_id),
        )
        .await;

        self.current_state = Some(WorkerState {
            guild_id: req.guild_id,
            scripts_vm: vm_cmd_tx,
            evt_rx: vm_evt_rx,
            shutdown_handle: vmthread,
            session_id: req.session_id,
        });

        self.write_message(WorkerMessage::Ack(req.seq)).await?;
        Ok(ContinueState::Continue)
    }

    async fn write_message(&mut self, v: WorkerMessage) -> anyhow::Result<()> {
        if self.scheduler_tx.send(v).is_err() {
            Err(anyhow::anyhow!("scheduler tx closed"))
        } else {
            Ok(())
        }
    }

    async fn wait_shutdown_current_vm(&mut self) -> anyhow::Result<()> {
        let Some(current) = self.current_state.take() else {
            return Ok(());
        };

        current
            .shutdown_handle
            .shutdown_vm(vm::vm::ShutdownReason::Request, false);

        current.scripts_vm.closed().await;

        self.handle_vm_channel_closed(&current).await
    }

    async fn handle_vm_channel_closed(&mut self, state: &WorkerState) -> anyhow::Result<()> {
        self.drain_runtime_events().await?;

        let reason = state.shutdown_handle.read_shutdown_reason();

        self.write_message(WorkerMessage::Shutdown(VmSessionShutdownEvent {
            reason,
            vm_session_id: state.session_id,
        }))
        .await
    }

    async fn drain_runtime_events(&mut self) -> anyhow::Result<()> {
        while let Ok(evt) = self.runtime_evt_rx.try_recv() {
            self.handle_runtime_evt(evt).await?;
        }

        Ok(())
    }

    fn current_guild_id(&self) -> Option<Id<GuildMarker>> {
        self.current_state.as_ref().map(|v| v.guild_id)
    }

    fn span_guild_id_or_scheduler_message_guild_id(
        &self,
        cmd: &SchedulerMessage,
    ) -> Option<Id<GuildMarker>> {
        cmd.guild_id().or(self.current_guild_id())
    }
}

enum ContinueState {
    Stop,
    Continue,
}

struct GuildLogForwarder {
    tx: mpsc::UnboundedSender<WorkerMessage>,
}

#[async_trait::async_trait]
impl guild_logger::GuildLoggerBackend for GuildLogForwarder {
    async fn handle_entry(&self, entry: guild_logger::LogEntry) {
        let _ = self.tx.send(WorkerMessage::GuildLog(entry));
    }
}

#[cfg(target_family = "unix")]
async fn connect_scheduler(
    path: &str,
    id: u64,
) -> (
    mpsc::UnboundedSender<WorkerMessage>,
    mpsc::UnboundedReceiver<SchedulerMessage>,
) {
    let mut stream = tokio::net::UnixStream::connect(path)
        .await
        .expect("scheduler should have opened socket");

    simpleproto::write_message(&WorkerMessage::Hello(id), &mut stream)
        .await
        .expect("should write to scheduler successfully");

    let (mut reader_half, mut writer_half) = stream.into_split();

    let scheduler_rx = {
        let (tx, rx) = mpsc::unbounded_channel::<SchedulerMessage>();

        tokio::spawn(async move { simpleproto::message_reader(&mut reader_half, tx).await });
        rx
    };

    let scheduler_tx = {
        let (tx, rx) = mpsc::unbounded_channel::<WorkerMessage>();
        tokio::spawn(async move { simpleproto::message_writer(&mut writer_half, rx).await });

        tx
    };

    (scheduler_tx, scheduler_rx)
}

#[cfg(target_family = "windows")]
async fn connect_scheduler(
    addr: &str,
    id: u64,
) -> (
    mpsc::UnboundedSender<WorkerMessage>,
    mpsc::UnboundedReceiver<SchedulerMessage>,
) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("scheduler should have opened socket");

    simpleproto::write_message(&WorkerMessage::Hello(id), &mut stream)
        .await
        .expect("should write to scheduler successfully");

    let (mut reader_half, mut writer_half) = stream.into_split();

    let scheduler_rx = {
        let (tx, rx) = mpsc::unbounded_channel::<SchedulerMessage>();

        tokio::spawn(async move { simpleproto::message_reader(&mut reader_half, tx).await });
        rx
    };

    let scheduler_tx = {
        let (tx, rx) = mpsc::unbounded_channel::<WorkerMessage>();
        tokio::spawn(async move { simpleproto::message_writer(&mut writer_half, rx).await });

        tx
    };

    (scheduler_tx, scheduler_rx)
}
