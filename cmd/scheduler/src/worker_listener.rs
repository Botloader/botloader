use scheduler_worker_rpc::WorkerMessage;
use tracing::error;

use crate::vmworkerpool::VmWorkerPool;

pub async fn listen_for_workers(path_or_addr: &str, worker_pool: VmWorkerPool) {
    #[cfg(target_family = "unix")]
    let listener = {
        let _ = std::fs::remove_file(path_or_addr);
        tokio::net::UnixListener::bind(path_or_addr).unwrap()
    };

    #[cfg(target_family = "windows")]
    let listener = tokio::net::TcpListener::bind(path_or_addr).await.unwrap();

    tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let cloned_pool = worker_pool.clone();

            tokio::spawn(async move {
                match simpleproto::read_message(&mut stream).await {
                    Ok(WorkerMessage::Hello(id)) => {
                        cloned_pool.worker_connected(stream, id);
                    }
                    Ok(_) => {
                        error!("first worker mesasge not hello");
                    }
                    Err(err) => {
                        error!(%err, "failed reading worker message");
                    }
                }
            });
        }
    });
}
