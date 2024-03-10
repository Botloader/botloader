use std::time::Duration;

use dbrokerapi::broker_scheduler_rpc::{BrokerEvent, SchedulerEvent};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tracing::{info, instrument, warn};

use crate::scheduler::SchedulerCommand;

pub async fn broker_client(addr: String, scheduler_tx: UnboundedSender<SchedulerCommand>) {
    loop {
        if scheduler_tx.is_closed() {
            return;
        }

        if let Ok(conn) = TcpStream::connect(&addr).await {
            info!("connected to broker");
            let client = BrokerConn {
                scheduler_tx: scheduler_tx.clone(),
                stream: conn,
            };
            let dc = client.run().await;
            info!("disconnected from broker: {:?}", dc);
            let _ = scheduler_tx.send(SchedulerCommand::BrokerDisconnected);
        } else {
            warn!("failed connecting to broker, retrying in a second");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

struct BrokerConn {
    stream: TcpStream,
    scheduler_tx: UnboundedSender<SchedulerCommand>,
}

enum ContinueState {
    Continue,
    Stop,
}

impl BrokerConn {
    async fn run(mut self) -> std::io::Result<()> {
        let _ = self.scheduler_tx.send(SchedulerCommand::BrokerConnected);

        loop {
            let next: BrokerEvent = simpleproto::read_message(&mut self.stream).await?;
            match self.handle_broker_message(next).await? {
                ContinueState::Continue => {}
                ContinueState::Stop => return Ok(()),
            }
        }
    }

    #[instrument(skip(self, message))]
    async fn handle_broker_message(
        &mut self,
        message: BrokerEvent,
    ) -> std::io::Result<ContinueState> {
        match message {
            BrokerEvent::Hello(h) => {
                if self
                    .scheduler_tx
                    .send(SchedulerCommand::BrokerHello(h))
                    .is_err()
                {
                    // return, close the connection, the broker will add it back to the queue
                    return Ok(ContinueState::Stop);
                }
            }
            BrokerEvent::DiscordEvent(evt) => {
                if self
                    .scheduler_tx
                    .send(SchedulerCommand::DiscordEvent(evt))
                    .is_err()
                {
                    // return, close the connection, the broker will add it back to the queue
                    return Ok(ContinueState::Stop);
                }
            }
        }

        // send ack
        simpleproto::write_message(&SchedulerEvent::Ack, &mut self.stream).await?;

        Ok(ContinueState::Continue)
    }
}
