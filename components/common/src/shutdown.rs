use std::{future::Future, task::Poll};
use tracing::info;
#[cfg(target_os = "linux")]
pub async fn wait_shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    info!("to shutodwn issue a sigint or sigterm");
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut terminate = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = interrupt.recv() =>{},
        _ = terminate.recv() =>{},
    }

    info!("got shutdown signal, shutting down...");
}

// TODO: implement this for other platforms
// realistically though, this wont be run in production outside linux, so it is needed?
#[cfg(not(target_os = "linux"))]
pub async fn wait_shutdown_signal() {
    info!("custom signal handling not implemented for this target, graceful shutdown disabled");
    Empty.await
}

/// A future which is never resolved.
struct Empty;

impl Future for Empty {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        Poll::Pending
    }
}
