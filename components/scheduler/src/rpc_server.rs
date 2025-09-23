use std::{sync::Arc, time::Instant};

use botrpc::{
    types::{GuildSpecifier, GuildStatusResponse, VmWorkerStatus, VmWorkerStatusResponse},
    BotServiceServer,
};
use guild_logger::guild_subscriber_backend::GuildSubscriberBackend;
use reqwest::StatusCode;
use service::{Empty, Json, SseStream};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::info;

use crate::scheduler::SchedulerCommand;

pub struct Server {
    addr: String,
    log_subscriber: Arc<GuildSubscriberBackend>,
    scheduler_tx: UnboundedSender<SchedulerCommand>,
}

impl Server {
    pub fn new(
        log_subscriber: Arc<GuildSubscriberBackend>,
        scheduler_tx: UnboundedSender<SchedulerCommand>,
        addr: String,
    ) -> Self {
        Self {
            log_subscriber,
            addr,
            scheduler_tx,
        }
    }

    pub async fn run(self) {
        let addr = self.addr.clone();

        let routes = botrpc::router(Arc::new(self));
        info!("starting bot rpc server on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, routes)
            .await
            .expect("failed starting botrpc");
    }
}

impl BotServiceServer for Server {
    async fn reload_vm(&self, payload: GuildSpecifier) -> Result<Empty, StatusCode> {
        let guild_id = payload.guild_id;

        let _ = self
            .scheduler_tx
            .send(SchedulerCommand::ReloadGuildScripts(guild_id));

        Ok(Empty)
    }

    async fn purge_guild_cache(&self, payload: GuildSpecifier) -> Result<Empty, StatusCode> {
        let guild_id = payload.guild_id;

        let _ = self
            .scheduler_tx
            .send(SchedulerCommand::PurgeGuildCache(guild_id));

        Ok(Empty)
    }

    async fn vm_worker_status(&self) -> Result<Json<VmWorkerStatusResponse>, StatusCode> {
        let (sender, receiver) = oneshot::channel();
        self.scheduler_tx
            .send(SchedulerCommand::WorkerStatus(sender))
            .unwrap();

        let result = receiver.await.unwrap();

        let now = Instant::now();
        Ok(Json(VmWorkerStatusResponse {
            workers: result
                .into_iter()
                .map(|v| VmWorkerStatus {
                    worker_id: v.worker_id as u32,
                    currently_claimed_by_guild_id: v.currently_claimed_by.map(|v| v.get()),
                    last_claimed_by_guild_id: v.claimed_last_by.map(|v| v.get()),
                    claimed_last_ms_ago: now.duration_since(v.claimed_last).as_millis() as u64,
                    returned_last_ms_ago: now.duration_since(v.returned_last).as_millis() as u64,
                })
                .collect(),
        }))
    }

    async fn guild_status(
        &self,
        payload: GuildSpecifier,
    ) -> Result<Json<GuildStatusResponse>, StatusCode> {
        let guild_id = payload.guild_id;

        let (sender, receiver) = oneshot::channel();
        self.scheduler_tx
            .send(SchedulerCommand::GuildStatus(guild_id, sender))
            .unwrap();

        if let Some(status) = receiver.await.unwrap() {
            let now = Instant::now();
            Ok(Json(GuildStatusResponse {
                current_claimed_worker_id: status.vm.current_claimed_worker.map(|v| v as u32),
                last_claimed_worker_id: status.vm.last_claimed_worker.map(|v| v as u32),
                claimed_last_since_ms: now.duration_since(status.vm.claimed_worker_at).as_millis()
                    as u64,
                returned_last_since_ms: now.duration_since(status.vm.returned_worker_at).as_millis()
                    as u64,
                pending_acks: status.vm.num_pending_acks as u32,
            }))
        } else {
            // Err(Status::not_found("guild not found"))
            Err(StatusCode::NOT_FOUND)
        }
    }

    async fn stream_guild_logs(
        &self,
        payload: GuildSpecifier,
    ) -> Result<service::SseStream<guild_logger::LogEntry>, StatusCode> {
        let guild_id = payload.guild_id;

        let mut rx = self.log_subscriber.subscribe(guild_id);
        let out = async_stream::try_stream! {
            while let Ok(next) = rx.recv().await{
                yield next;
            }
        };

        Ok(SseStream::new(out))
    }
}
