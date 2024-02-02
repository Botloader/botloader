use std::{rc::Rc, time::Duration};

use isolatecell::IsolateCell;
use metrics::counter;
use tokio::{
    sync::{mpsc, oneshot},
    task::LocalSet,
    time::Instant,
};
use tracing::Instrument;

use crate::vm::{CreateRt, ShutdownReason, Vm, VmShutdownHandle};

pub async fn spawn_vm_thread<F: FnOnce() -> tracing::Span + Send + Sync + 'static>(
    create: CreateRt,
    make_span: F,
) -> VmShutdownHandle {
    let (vm_created_send, vm_created_recv) = oneshot::channel();
    let (ping_send, mut ping_recv) = mpsc::channel::<oneshot::Sender<()>>(1);

    std::thread::spawn(move || {
        let iso_cell = Rc::new(IsolateCell::new_with_tracker(Box::new(|dur| {
            counter!("bl.vm.cpu_microseconds_total").increment(dur.as_micros() as u64);
        })));

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let result = Vm::create_with_handles(create, iso_cell);
        vm_created_send
            .send(result.shutdown_handle.clone())
            .unwrap();

        // tokio_current.block_on(t);
        let span = make_span();
        rt.block_on(
            async move {
                let set = LocalSet::new();

                // A simple task that sends echo responses on a channel
                //
                // if the js runtime encounters a runaway situation with something like
                // an infinite "for" loop, then the thread will be blocked and this task
                // will not send echo responses anymore, leading to an outside thread
                // being able to detect the runaway and being able to shut down the runaway runtime
                set.spawn_local(async move {
                    loop {
                        match ping_recv.recv().await {
                            Some(r) => {
                                let _ = r.send(());
                            }
                            None => return,
                        }
                    }
                });
                set.run_until(result.future).await
            }
            .instrument(span),
        );
    });

    let shutdown_handle = vm_created_recv.await.unwrap();

    tokio::spawn(monitor_vm_runaway(shutdown_handle.clone(), ping_send));

    shutdown_handle
}

// runaway script detection ensures that no single vm can
// block the thread for more than the allowed interval
async fn monitor_vm_runaway(
    shutdown_handle: VmShutdownHandle,
    ping_send: mpsc::Sender<oneshot::Sender<()>>,
) {
    let ping_interval = Duration::from_secs(10);
    loop {
        let (send, rcv) = oneshot::channel();
        match ping_send.send(send).await {
            Ok(_) => {
                let last_ping = Instant::now();
                match tokio::time::timeout(ping_interval, rcv).await {
                    Ok(_) => {
                        // sleep until the next ping
                        let remaining = ping_interval - last_ping.elapsed();
                        tokio::time::sleep(remaining).await;
                    }
                    Err(_) => {
                        // we hit a timeout, meaning there's a runaway script
                        //
                        // note that this logic is currently very flawed,
                        // you could make a vm that takes a 1 less millisecond than the ping interval
                        // then the next vm takes move than 1 millisecond and this logic
                        // will think the cause is the latter even though it was only responsible for 1ms
                        //
                        // in the future we will have to actively track the cpu usage of vm's to shut down the proper vm
                        // if there's a bad actor
                        shutdown_handle.shutdown_vm(ShutdownReason::Runaway, true);
                    }
                }
            }
            Err(_) => {
                // receiver was dropped meaning the thread has shut down
                return;
            }
        }
    }
}
