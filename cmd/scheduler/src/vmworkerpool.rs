use std::{
    collections::{HashMap, VecDeque},
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use metrics::counter;
use simpleproto::{message_reader, message_writer};
use stores::config::PremiumSlotTier;
use tokio::{
    net::UnixStream,
    process::{Child, Command},
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};
use tracing::{error, info};
use twilight_model::id::{marker::GuildMarker, Id};

pub enum WorkerRetrieved {
    SameGuild,
    OtherGuild,
}

struct PoolInner {
    worker_id_gen: u64,

    pending_starts: HashMap<u64, PendingWorkerHandle>,

    pools: [Vec<WorkerHandle>; MAX_PREMIUM_SLOT_TIER + 1],
    req_queues: [VecDeque<oneshot::Sender<WorkerHandle>>; MAX_PREMIUM_SLOT_TIER + 1],
}

#[derive(Clone)]
pub struct VmWorkerPool {
    inner: Arc<Mutex<PoolInner>>,
    launch_config: WorkerLaunchConfig,
}

impl VmWorkerPool {
    pub fn new(launch_config: WorkerLaunchConfig) -> VmWorkerPool {
        Self {
            inner: Arc::new(Mutex::new(PoolInner {
                pools: Default::default(),
                req_queues: Default::default(),
                worker_id_gen: 1,
                pending_starts: HashMap::new(),
            })),
            launch_config,
        }
    }

    pub async fn req_worker(
        &self,
        guild_id: Id<GuildMarker>,
        premium_tier: Option<PremiumSlotTier>,
    ) -> (WorkerHandle, WorkerRetrieved) {
        let mut worker = self.inner_get_worker(guild_id, premium_tier).await;
        let wr = if matches!(worker.last_active_guild, Some(g) if g == guild_id) {
            WorkerRetrieved::SameGuild
        } else {
            WorkerRetrieved::OtherGuild
        };

        worker.claim(guild_id);
        (worker, wr)
    }

    async fn inner_get_worker(
        &self,
        guild_id: Id<GuildMarker>,
        premium_tier: Option<PremiumSlotTier>,
    ) -> WorkerHandle {
        let rx = {
            let mut w = self.inner.lock().unwrap();

            let priority_index = premium_tier_index(premium_tier);

            // try to find one with identical guild id, avoids us having to reload all scripts
            let mut i = priority_index;
            loop {
                let pool = &mut w.pools[i];
                let pref_worker = pool
                    .iter()
                    .enumerate()
                    .find(|(_, v)| matches!(v.last_active_guild, Some(g) if g == guild_id))
                    .map(|(i, _)| i);

                if let Some(pref_worker) = pref_worker {
                    metrics::gauge!("bl.scheduler.workerpool_available_workers", "priority_index" => i.to_string()).decrement(1.0);
                    return pool.remove(pref_worker);
                }

                if i == 0 {
                    break;
                }

                i -= 1;
            }

            // take the least recently used worker
            let mut i = priority_index;
            loop {
                let pool = &mut w.pools[i];

                let mut candidate = None;
                let mut canditate_age = Duration::MAX;
                for (i, worker) in pool.iter().enumerate() {
                    let elapsed = Instant::elapsed(&worker.returned_at);
                    if candidate.is_none() || elapsed > canditate_age {
                        candidate = Some(i);
                        canditate_age = elapsed;
                    }
                }

                if let Some(can) = candidate {
                    metrics::gauge!("bl.scheduler.workerpool_available_workers", "priority_index" => i.to_string()).decrement(1.0);
                    return pool.remove(can);
                }

                if i == 0 {
                    break;
                }

                i -= 1;
            }

            // no available workers, queue the request
            let (tx, rx) = oneshot::channel();
            let queue = &mut w.req_queues[priority_index];
            queue.push_back(tx);
            rx
        };

        // the tx end should never be dropped
        rx.await.unwrap()
    }

    pub fn spawn_workers(&self, tier: Option<PremiumSlotTier>, n: usize) {
        for _ in 0..n {
            self.spawn_worker(premium_tier_index(tier));
        }
    }

    pub fn return_worker(&self, mut worker: WorkerHandle, broken: bool) {
        worker.returned_at = Instant::now();
        let elapsed = worker.returned_at - worker.claimed_at;
        if let Some(guild_id) = worker.last_active_guild {
            let micros = elapsed.as_micros() as u64;
            counter!("bl.worker.claimed_microseconds_total", "guild_id" => guild_id.get().to_string()).increment(micros);
        }

        if broken {
            error!(
                "returned broken worker to the pool: {:?}",
                worker.last_active_guild
            );
            metrics::counter!("bl.scheduler.broken_workers_total").increment(1);
            self.spawn_worker(worker.priority_index);
        } else {
            info!(
                tier = worker.priority_index,
                dur = elapsed.as_secs_f64(),
                "returned worker to the pool"
            );
            self.add_worker_to_pool(worker);
        }
    }

    fn add_worker_to_pool(&self, worker: WorkerHandle) {
        let mut w = self.inner.lock().unwrap();

        // potentially hand over to next queued request
        let mut i = MAX_PREMIUM_SLOT_TIER;
        loop {
            let queue = &mut w.req_queues[i];
            if let Some(tx) = queue.pop_front() {
                if tx.send(worker).is_err() {
                    panic!("worker request dropped")
                }

                return;
            }

            if i == worker.priority_index {
                break;
            }

            i -= 1;
        }

        // no pending worker requests
        metrics::gauge!("bl.scheduler.workerpool_available_workers", "priority_index" => worker.priority_index.to_string()).increment(1.0);
        w.pools[worker.priority_index].push(worker);
    }

    fn add_pending_worker(&self, worker: PendingWorkerHandle) {
        let mut w = self.inner.lock().unwrap();
        w.pending_starts.insert(worker.worker_id, worker);
    }

    fn gen_id(&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        inner.worker_id_gen += 1;
        inner.worker_id_gen
    }

    fn spawn_worker(&self, tier: usize) {
        info!("spawning vm worker");

        let mut cmd = Command::new(&self.launch_config.cmd);
        // cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        cmd.envs(std::env::vars());
        let worker_id = self.gen_id();
        cmd.env("BL_WORKER_ID", worker_id.to_string());

        let child = cmd.spawn().expect("spawn script vm worker");
        // let handle = init_worker_handles(child);

        // self.self.add_worker_to_pool(handle);
        self.add_pending_worker(PendingWorkerHandle {
            child,
            worker_id,
            priority_index: tier,
        })
    }

    pub fn worker_connected(&self, stream: UnixStream, id: u64) {
        let mut w = self.inner.lock().unwrap();
        if let Some(pending) = w.pending_starts.remove(&id) {
            drop(w);

            let full = init_worker_handles(pending, stream);
            info!(tier = full.priority_index, "worker connected");
            self.add_worker_to_pool(full);
        }
    }
}

pub struct WorkerHandle {
    pub child: Child,
    pub tx: UnboundedSender<scheduler_worker_rpc::SchedulerMessage>,
    pub rx: UnboundedReceiver<scheduler_worker_rpc::WorkerMessage>,
    last_active_guild: Option<Id<GuildMarker>>,
    pub returned_at: Instant,
    pub claimed_at: Instant,
    pub worker_id: u64,
    pub priority_index: usize,
}

impl WorkerHandle {
    fn claim(&mut self, guild_id: Id<GuildMarker>) {
        self.last_active_guild = Some(guild_id);
        self.claimed_at = Instant::now();
    }
}

struct PendingWorkerHandle {
    child: Child,
    worker_id: u64,
    priority_index: usize,
}

fn init_worker_handles(pending: PendingWorkerHandle, stream: UnixStream) -> WorkerHandle {
    let (scheduler_msg_tx, scheduler_msg_rx) = mpsc::unbounded_channel();
    let (worker_msg_tx, worker_msg_rx) = mpsc::unbounded_channel();

    let (mut reader, mut writer) = stream.into_split();

    tokio::spawn(async move { message_reader(&mut reader, worker_msg_tx).await });

    tokio::spawn(async move { message_writer(&mut writer, scheduler_msg_rx).await });

    WorkerHandle {
        child: pending.child,
        tx: scheduler_msg_tx,
        rx: worker_msg_rx,

        last_active_guild: None,
        returned_at: Instant::now(),
        claimed_at: Instant::now(),
        worker_id: pending.worker_id,
        priority_index: pending.priority_index,
    }
}

#[derive(Clone)]
pub struct WorkerLaunchConfig {
    pub cmd: String,
}

const MAX_PREMIUM_SLOT_TIER: usize = 2;

fn premium_tier_index(tier: Option<PremiumSlotTier>) -> usize {
    match tier {
        None => 0,
        Some(PremiumSlotTier::Lite) => 1,
        Some(PremiumSlotTier::Premium) => 2,
    }
}
