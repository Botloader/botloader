use std::{collections::HashMap, ops::Add, str::FromStr, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use stores::timers::{IntervalTimer, IntervalType, TimerStore};
use tokio::sync::mpsc;
use twilight_model::id::GuildId;
use vm::vm::VmCommand;

#[derive(Debug)]
pub enum Error {
    StorageError(anyhow::Error),
    CronParseError(cron::error::Error),
    NoNextTime,
}

pub struct ScriptStartedCommand {
    pub guild_id: GuildId,
    pub timers: Vec<stores::config::IntervalTimerContrib>,

    pub dispath_tx: mpsc::UnboundedSender<VmCommand>,
    pub script_id: u64,
}

pub enum Command {
    ScriptStarted(ScriptStartedCommand),
}

pub struct Manager {
    storage: Arc<dyn TimerStore>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,

    guild_id: GuildId,
    dispatch_tx: mpsc::UnboundedSender<VmCommand>,

    loaded_intervals: HashMap<String, WrappedIntervalTimer>,
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
            loaded_intervals: HashMap::new(),
        };

        tokio::task::spawn_local(async move { scheduler.run().await });

        tx
    }

    pub async fn run(mut self) {
        loop {
            if let Some(next) = self.next_event_time() {
                let to_sleep = next - chrono::Utc::now();
                if to_sleep > Duration::seconds(0) {
                    tokio::select! {
                        _ = tokio::time::sleep(to_sleep.to_std().unwrap()) => {
                            self.check_run_next_timer().await;
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
                    self.check_run_next_timer().await;
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

    async fn check_run_next_timer(&mut self) {
        let now = chrono::Utc::now();
        let triggered_timers = self
            .loaded_intervals
            .iter()
            .filter(|(_, t)| t.is_triggered(now))
            .map(|(_, t)| t.clone())
            .collect::<Vec<_>>();

        for triggered in triggered_timers {
            self.trigger_timer(now, triggered).await;
        }
    }

    async fn trigger_timer(&mut self, t: DateTime<Utc>, timer: WrappedIntervalTimer) {
        let evt = runtime_models::events::timers::IntervalTimerEvent {
            name: timer.inner.name.clone(),
        };

        let serialized = serde_json::to_value(&evt).unwrap();

        // if this fails, were probably returning soon anyways from our future being dropped from the local set
        self.dispatch_tx
            .send(VmCommand::DispatchEvent(
                "BOTLOADER_INTERVAL_TIMER_FIRED",
                serialized,
            ))
            .ok();

        self.update_next_run(t, timer.inner.name.clone()).await;
    }

    async fn update_next_run(&mut self, t: DateTime<Utc>, name: String) {
        let timer = if let Some(timer) = self.loaded_intervals.get_mut(&name) {
            timer
        } else {
            return;
        };

        timer.inner.last_run = t;
        if let Some(next) = timer.parsed_type.next_run_time(t) {
            timer.next_run = next;
        } else {
            timer.next_run = t.add(Duration::hours(1000));
        }

        // update last run
        if let Err(err) = self
            .storage
            .update_interval_timer(self.guild_id, timer.inner.clone())
            .await
        {
            tracing::error!(%err, "failed updating timer")
        };
    }

    async fn handle_command(&mut self, cmd: Command) {
        let res = match cmd {
            Command::ScriptStarted(g) => self.script_started(g).await,
        };

        if let Err(err) = res {
            tracing::error!(%err, "failed syncing guild");
        }
    }

    async fn script_started(&mut self, cmd: ScriptStartedCommand) -> Result<(), anyhow::Error> {
        let all_guild_timers = self.storage.get_all_interval_timers(self.guild_id).await?;

        for updt in cmd.timers {
            let last_run = all_guild_timers
                .iter()
                .find(|v| v.name == updt.name)
                .map(|v| v.last_run)
                .unwrap_or_else(chrono::Utc::now);

            let timer = self
                .storage
                .update_interval_timer(
                    self.guild_id,
                    IntervalTimer {
                        last_run,
                        interval: updt.interval,
                        name: updt.name,
                    },
                )
                .await?;

            match wrap_timer(timer) {
                Ok(wrapped) => {
                    self.loaded_intervals
                        .insert(wrapped.inner.name.clone(), wrapped);
                }
                Err(err) => tracing::error!(?err, "failed wrapping timer"),
            };
        }

        Ok(())
    }

    pub fn next_event_time(&self) -> Option<DateTime<Utc>> {
        let lowest_interval = self
            .loaded_intervals
            .iter()
            .min_by(|(_, a), (_, b)| a.next_run.cmp(&b.next_run));

        lowest_interval.map(|(_, v)| v.next_run)
    }
}

#[derive(Clone)]
pub struct WrappedIntervalTimer {
    inner: IntervalTimer,
    parsed_type: ParsedIntervalType,
    next_run: chrono::DateTime<chrono::Utc>,
}

fn wrap_timer(timer: IntervalTimer) -> Result<WrappedIntervalTimer, Error> {
    let interval_type = match &timer.interval {
        IntervalType::Minutes(mins) => ParsedIntervalType::Minutes(*mins),
        IntervalType::Cron(c) => {
            let parsed = cron::Schedule::from_str(format!("0 {}", c).as_str())
                .map_err(Error::CronParseError)?;
            ParsedIntervalType::Cron(c.clone(), Box::new(parsed))
        }
    };

    let next = if let Some(next) = interval_type.next_run_time(timer.last_run) {
        next
    } else {
        return Err(Error::NoNextTime);
    };

    Ok(WrappedIntervalTimer {
        inner: timer,
        next_run: next,
        parsed_type: interval_type,
    })
}

impl WrappedIntervalTimer {
    fn is_triggered(&self, t: DateTime<Utc>) -> bool {
        t > self.next_run
    }
}

#[derive(Clone, PartialEq)]
pub enum ParsedIntervalType {
    Minutes(u64),
    Cron(String, Box<cron::Schedule>),
}

impl ParsedIntervalType {
    fn next_run_time(&self, t: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Self::Cron(_, c) => c.after(&t).next(),
            Self::Minutes(minutes) => Some(t.add(chrono::Duration::minutes(*minutes as i64))),
        }
    }
}
