use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{error, info, warn};

pub type OutputFuture = Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>;

pub trait Job: Send + Sync {
    fn status(&self) -> String;
    fn run(self: Arc<Self>) -> OutputFuture;
}

pub trait JobSpawner: Send + Sync {
    fn name(&self) -> &'static str;
    fn spawn(&self) -> Arc<dyn Job>;
    fn interval(&self) -> std::time::Duration;
}

pub struct JobRunner {
    jobs: Mutex<Vec<JobSlot>>,
    completed_jobs_tx: UnboundedSender<(String, Result<(), anyhow::Error>)>,
}

impl JobRunner {
    pub async fn new_run(
        jobs: Vec<Box<dyn JobSpawner>>,
        shutdown_future: impl Future<Output = ()> + Send + Sync + 'static,
    ) {
        let (tx, rx) = unbounded_channel();

        let res = Arc::new(Self {
            completed_jobs_tx: tx,
            jobs: Mutex::new(jobs.into_iter().map(|job| JobSlot::new(job)).collect()),
        });

        tokio::spawn(res.clone().run_completed_jobs_listener(rx));

        res.run_jobs(shutdown_future).await;
    }

    async fn run_jobs(self: Arc<Self>, shutdown_future: impl Future<Output = ()> + Send + Sync) {
        tokio::pin!(shutdown_future);

        loop {
            self.trigger_jobs().await;

            tokio::select! {
                _ = &mut shutdown_future => {
                    break;
                },
                _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                    continue;
                }
            }
        }

        self.complete_jobs().await
    }

    async fn complete_jobs(&self) {
        loop {
            let mut remaining_jobs = 0;
            {
                let mut jobs = self.jobs.lock().unwrap();

                for job in &mut *jobs {
                    if job.is_running() {
                        remaining_jobs += 1
                    }
                }
            }

            if remaining_jobs >= 1 {
                warn!("Waiting on {remaining_jobs} jobs to complete");
            } else {
                return;
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await
        }
    }

    async fn run_completed_jobs_listener(
        self: Arc<Self>,
        mut rx: UnboundedReceiver<(String, Result<(), anyhow::Error>)>,
    ) {
        loop {
            match rx.recv().await {
                Some((name, result)) => {
                    self.job_completed(name, result);
                }
                None => {
                    return;
                }
            }
        }
    }

    fn job_completed(&self, name: String, result: Result<(), anyhow::Error>) {
        let mut jobs = self.jobs.lock().unwrap();

        for job in &mut *jobs {
            if job.name() == name {
                job.complete(result);
                return;
            }
        }

        panic!("Unknown job {name}");
    }

    async fn trigger_jobs(&self) {
        let now = Utc::now();

        let mut jobs = self.jobs.lock().unwrap();
        for job in &mut *jobs {
            let next = job.next_run(now);
            if now >= next {
                if job.run(self.completed_jobs_tx.clone()) {
                    info!("Running job {}", job.name());
                }
            }
        }
    }
}

pub struct JobSlot {
    spawner: Box<dyn JobSpawner>,
    last_run: Option<chrono::DateTime<Utc>>,

    last_started: Option<chrono::DateTime<Utc>>,
    last_completed: Option<chrono::DateTime<Utc>>,
    job: Option<Arc<dyn Job>>,
    last_result: Result<(), anyhow::Error>,
}

impl JobSlot {
    fn new(spawner: Box<dyn JobSpawner>) -> Self {
        Self {
            spawner: spawner,
            last_run: None,
            last_started: None,
            last_completed: None,
            job: None,
            last_result: Ok(()),
        }
    }

    pub fn is_running(&self) -> bool {
        self.job.is_some()
    }

    pub fn name(&self) -> &'static str {
        self.spawner.name()
    }

    pub fn running_status(&self) -> Option<String> {
        self.job.as_ref().map(|v| v.status().to_string())
    }

    pub fn complete(&mut self, result: Result<(), anyhow::Error>) {
        self.job = None;
        let now = Utc::now();
        self.last_completed = Some(now);

        let elapsed = self
            .last_started
            .map(|v| now.signed_duration_since(v))
            .unwrap_or_default();

        match &result {
            Ok(_) => {
                info!("Completed job {} successfully in {}", self.name(), elapsed);
            }
            Err(err) => {
                error!("Failed job {} in {}: {}", self.name(), elapsed, err);
            }
        }

        self.last_result = result;
    }

    pub fn next_run(&self, t: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
        if let Some(last) = &self.last_run {
            return *last + chrono::Duration::from_std(self.spawner.interval()).unwrap();
        }

        t
    }

    pub fn run(&mut self, tx: UnboundedSender<(String, Result<(), anyhow::Error>)>) -> bool {
        if self.is_running() {
            return false;
        }

        self.last_run = Some(Utc::now());
        let spawned = self.spawner.spawn();
        let spawned_clone = spawned.clone();

        let name = self.name().to_string();
        tokio::spawn(async move {
            let res = spawned_clone.run().await;
            tx.send((name, res)).unwrap();
        });

        self.job = Some(spawned);

        true
    }
}

pub enum JobStatus {
    Inactive,
    Running(String),
}
