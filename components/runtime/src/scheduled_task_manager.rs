use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use stores::timers::{ScheduledTask, TimerStore};
use tokio::sync::mpsc;
use tracing::{error, info};
use twilight_model::id::GuildId;
use vm::vm::VmCommand;

pub enum Command {
    RefreshNext,
}

pub struct Manager {
    storage: Arc<dyn TimerStore>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,

    guild_id: GuildId,
    dispatch_tx: mpsc::UnboundedSender<VmCommand>,
}

impl Manager {
    pub fn create(
        guild_id: GuildId,
        dispatch_tx: mpsc::UnboundedSender<VmCommand>,
        storage: Arc<dyn TimerStore>,
    ) -> mpsc::UnboundedSender<Command> {
        let (tx, rx) = mpsc::unbounded_channel();

        let scheduler = Self {
            storage,
            cmd_rx: rx,
            dispatch_tx,
            guild_id,
        };

        tokio::task::spawn_local(async move { scheduler.run().await });

        tx
    }

    pub async fn run(mut self) {
        loop {
            if let Some(next) = self.next_event_time().await {
                let to_sleep = next - chrono::Utc::now();
                if to_sleep > Duration::seconds(0) {
                    tokio::select! {
                        _ = tokio::time::sleep(to_sleep.to_std().unwrap()) => {
                            self.check_run_tasks().await;
                        },
                        evt = self.cmd_rx.recv() => {
                            if let Some(evt) = evt{
                                self.handle_command(evt).await;
                            }else{
                                return;
                            }
                        },
                    }
                } else {
                    self.check_run_tasks().await;
                }
            } else if !self.check_run_next_cmd().await {
                return;
            }
        }
    }

    async fn check_run_next_cmd(&mut self) -> bool {
        let cmd = self.cmd_rx.recv().await;
        if let Some(cmd) = cmd {
            self.handle_command(cmd).await;
            true
        } else {
            // commands sender end dropped, probably shutting down
            false
        }
    }

    async fn check_run_tasks(&mut self) {
        let now = chrono::Utc::now();

        let triggered =
            if let Ok(triggered) = self.storage.get_triggered_tasks(self.guild_id, now).await {
                triggered
            } else {
                return;
            };

        for t in triggered {
            self.trigger_task(t).await;
        }
    }

    async fn trigger_task(&mut self, task: ScheduledTask) {
        let evt = runtime_models::events::task::ScheduledTask::from(task);
        let serialized = serde_json::to_value(&evt).unwrap();

        info!("firing scheduled task {}", evt.namespace);

        // if this fails, were probably returning soon anyways from our future being dropped from the local set
        self.dispatch_tx
            .send(VmCommand::DispatchEvent(
                "BOTLOADER_SCHEDULED_TASK_FIRED",
                serialized,
            ))
            .ok();

        loop {
            match self.storage.del_task_by_id(self.guild_id, evt.id.0).await {
                Ok(_) => return,
                Err(err) => {
                    error!(%err, "failed deleting task");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            // nothing more needs to be done, in the next run() loop iteration the ne time is fetched
            Command::RefreshNext => {}
        };
    }

    pub async fn next_event_time(&self) -> Option<DateTime<Utc>> {
        loop {
            match self.storage.get_next_task_time(self.guild_id).await {
                Ok(v) => return v,
                Err(err) => {
                    error!(%err, "failed fetching next task time");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}
