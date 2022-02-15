use std::{pin::Pin, sync::Arc};

use futures::Stream;
use guild_logger::guild_subscriber_backend::GuildSubscriberBackend;
use tokio::sync::mpsc::UnboundedSender;
use tonic::{Response, Status};
use twilight_model::id::GuildId;

use botrpc::proto;

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
        let guild_id = GuildId::new(request.into_inner().guild_id);

        let _ = self
            .scheduler_tx
            .send(SchedulerCommand::ReloadGuildScripts(guild_id));

        Ok(Response::new(proto::Empty {}))
    }

    type StreamGuildLogsStream = ResponseStream;

    async fn stream_guild_logs(
        &self,
        request: tonic::Request<proto::GuildSpecifier>,
    ) -> Result<Response<Self::StreamGuildLogsStream>, Status> {
        let guild_id = GuildId::new(request.into_inner().guild_id);

        let mut rx = self.log_subscriber.subscribe(guild_id);
        let out = async_stream::try_stream! {
            while let Ok(next) = rx.recv().await{
                yield proto::GuildLogItem::from(next);
            }
        };

        Ok(Response::new(Box::pin(out)))
    }
}
