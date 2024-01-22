use std::{pin::Pin, sync::Arc, time::Instant};

use futures::Stream;
use guild_logger::guild_subscriber_backend::GuildSubscriberBackend;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tonic::{Response, Status};

use botrpc::proto;
use twilight_model::id::Id;

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
        tonic::transport::Server::builder()
            .add_service(proto::bot_service_server::BotServiceServer::new(self))
            .serve(addr.parse().unwrap())
            .await
            .expect("failed starting botrpc");
    }
}

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<proto::GuildLogItem, Status>> + Send + Sync>>;

#[tonic::async_trait]
impl proto::bot_service_server::BotService for Server {
    async fn reload_vm(
        &self,
        request: tonic::Request<proto::GuildScriptSpecifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        let guild_id = Id::new(request.into_inner().guild_id);

        let _ = self
            .scheduler_tx
            .send(SchedulerCommand::ReloadGuildScripts(guild_id));

        Ok(Response::new(proto::Empty {}))
    }

    async fn purge_guild_cache(
        &self,
        request: tonic::Request<proto::GuildScriptSpecifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        let guild_id = Id::new(request.into_inner().guild_id);

        let _ = self
            .scheduler_tx
            .send(SchedulerCommand::PurgeGuildCache(guild_id));

        Ok(Response::new(proto::Empty {}))
    }

    type StreamGuildLogsStream = ResponseStream;

    async fn stream_guild_logs(
        &self,
        request: tonic::Request<proto::GuildSpecifier>,
    ) -> Result<Response<Self::StreamGuildLogsStream>, Status> {
        let guild_id = Id::new(request.into_inner().guild_id);

        let mut rx = self.log_subscriber.subscribe(guild_id);
        let out = async_stream::try_stream! {
            while let Ok(next) = rx.recv().await{
                yield proto::GuildLogItem::from(next);
            }
        };

        Ok(Response::new(Box::pin(out)))
    }

    async fn vm_worker_status(
        &self,
        _request: tonic::Request<proto::Empty>,
    ) -> Result<Response<proto::VmWorkerStatusResponse>, Status> {
        let (sender, receiver) = oneshot::channel();
        self.scheduler_tx
            .send(SchedulerCommand::WorkerStatus(sender))
            .unwrap();

        let result = receiver.await.unwrap();

        let now = Instant::now();
        Ok(Response::new(proto::VmWorkerStatusResponse {
            workers: result
                .into_iter()
                .map(|v| proto::VmWorkerStatus {
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
        request: tonic::Request<proto::GuildSpecifier>,
    ) -> Result<Response<proto::GuildStatusResponse>, Status> {
        let guild_id = Id::new(request.into_inner().guild_id);

        let (sender, receiver) = oneshot::channel();
        self.scheduler_tx
            .send(SchedulerCommand::GuildStatus(guild_id, sender))
            .unwrap();

        if let Some(status) = receiver.await.unwrap() {
            let now = Instant::now();
            Ok(Response::new(proto::GuildStatusResponse {
                current_claimed_worker_id: status.vm.current_claimed_worker.map(|v| v as u32),
                last_claimed_worker_id: status.vm.last_claimed_worker.map(|v| v as u32),
                claimed_last_since_ms: now.duration_since(status.vm.claimed_worker_at).as_millis()
                    as u64,
                returned_last_since_ms: now.duration_since(status.vm.returned_worker_at).as_millis()
                    as u64,
                pending_acks: status.vm.num_pending_acks as u32,
            }))
        } else {
            Err(Status::not_found("guild not found"))
        }
    }
}
