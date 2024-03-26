use std::ops::Add;

use chrono::{DateTime, Utc};
use runtime_models::internal::script::{ScriptMeta, TaskBucketId};
use stores::{timers::ScheduledTask, Db};
use tracing::{error, info};
use twilight_model::id::{marker::GuildMarker, Id};

pub struct Manager {
    storage: Db,
    guild_id: Id<GuildMarker>,

    // outer option: none if not fetched, some if fetched
    // inner: none if no tasks remaining
    next_task_time: Option<Option<DateTime<Utc>>>,
    pending: Vec<u64>,

    active_task_buckets: Vec<TaskBucketId>,
}

impl Manager {
    pub fn new(guild_id: Id<GuildMarker>, storage: Db) -> Self {
        Self {
            storage,
            guild_id,
            next_task_time: None,
            pending: Vec::new(),
            active_task_buckets: Vec::new(),
        }
    }

    pub async fn init_next_task_time(&mut self) {
        if self.next_task_time.is_some() {
            return;
        }

        // fetch
        match self
            .storage
            .get_next_task_time(self.guild_id, &self.pending, &self.active_task_buckets)
            .await
        {
            Ok(v) => {
                self.next_task_time = Some(v);
            }
            Err(err) => {
                error!(%err, "failed fetching next task time");
                self.next_task_time = Some(Some(Utc::now().add(chrono::Duration::seconds(10))));
            }
        }
    }

    pub fn next_action(&mut self) -> NextAction {
        match self.next_task_time {
            None => NextAction::None,
            Some(None) => NextAction::None,
            Some(Some(t)) => {
                if Utc::now() > t {
                    NextAction::Run
                } else {
                    NextAction::Wait(t)
                }
            }
        }
    }

    pub async fn start_triggered_tasks(&mut self) -> Vec<ScheduledTask> {
        // trigger some tasks
        match self
            .storage
            .get_triggered_tasks(
                self.guild_id,
                Utc::now(),
                &self.pending,
                &self.active_task_buckets,
            )
            .await
        {
            Ok(v) => {
                for task in &v {
                    self.pending.push(task.id);
                }
                info!("pending tasks: {}", self.pending.len());
                self.clear_next();
                v
            }
            Err(err) => {
                error!(%err, "failed fetching triggered tasks time");
                Vec::new()
            }
        }
    }

    pub async fn ack_triggered_task(&mut self, id: u64) {
        if let Some(index) =
            self.pending
                .iter()
                .enumerate()
                .find_map(|(i, v)| if *v == id { Some(i) } else { None })
        {
            self.pending.swap_remove(index);
        }

        loop {
            match self.storage.del_task_by_id(self.guild_id, id).await {
                Ok(_) => return,
                Err(err) => {
                    error!(%err, "failed deleting task");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    // pub async fn failed_ack_pending(&mut self, id: u64) {
    //     if let Some(index) =
    //         self.pending
    //             .iter()
    //             .enumerate()
    //             .find_map(|(i, v)| if *v == id { Some(i) } else { None })
    //     {
    //         self.pending.swap_remove(index);
    //     }

    //     self.clear_next();
    // }

    pub fn clear_pending(&mut self) {
        info!("cleared pending");
        self.pending.clear();
    }

    pub fn clear_next(&mut self) {
        self.next_task_time = None;
    }

    pub fn clear_task_names(&mut self) {
        self.active_task_buckets.clear();
    }

    pub fn script_started(&mut self, meta: &ScriptMeta) {
        for script_task_bucket in &meta.task_buckets {
            if self.active_task_buckets.contains(script_task_bucket) {
                continue;
            }

            self.active_task_buckets.push(script_task_bucket.clone());
        }

        self.clear_next();
    }
}

pub type NextAction = crate::guild_handler::NextTimerAction;
