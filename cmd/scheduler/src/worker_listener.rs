use scheduler_worker_rpc::WorkerMessage;
use tokio::net::UnixStream;
use tracing::error;

use crate::vmworkerpool::VmWorkerPool;

pub fn listen_for_workers(path: &str, worker_pool: VmWorkerPool) {
    let _ = std::fs::remove_file(path);

    let listener = tokio::net::UnixListener::bind(path).unwrap();

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(handle_stream(stream, worker_pool.clone()));
        }
    });
}

async fn handle_stream(mut stream: UnixStream, worker_pool: VmWorkerPool) {
    match simpleproto::read_message(&mut stream).await {
        Ok(WorkerMessage::Hello(id)) => {
            worker_pool.worker_connected(stream, id);
        }
        Ok(_) => {
            error!("first worker mesasge not hello");
        }
        Err(err) => {
            error!(%err, "failed reading worker message");
        }
    }
}
