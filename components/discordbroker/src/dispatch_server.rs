use tokio::net::TcpListener;

use crate::broker::{BrokerCommand, BrokerHandle};

pub async fn start_server(addr: String, broker: BrokerHandle) {
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        socket.set_nodelay(true).unwrap();
        if broker
            .send(BrokerCommand::SchedulerConnected(socket))
            .is_err()
        {
            return;
        }
    }
}
