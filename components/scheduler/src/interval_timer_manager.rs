use std::{collections::HashMap, ops::Add, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use stores::{
    config::IntervalTimerContrib,
    timers::{IntervalTimer, IntervalType},
    Db,
};
use tracing::info;
use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Debug)]
pub enum Error {
    Cron(cron::error::Error),
    NoNextTime,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TimerId(Option<u64>, String);

impl TimerId {
    pub fn new(plugin_id: Option<u64>, name: String) -> Self {
        TimerId(plugin_id, name)
    }

    pub fn identifies_timer(&self, timer: &IntervalTimer) -> bool {
        timer.plugin_id == self.0 && timer.name == self.1
    }
}

pub struct Manager {
    storage: Db,
    guild_id: Id<GuildMarker>,
    loaded_intervals: HashMap<TimerId, WrappedIntervalTimer>,
    pending: Vec<IntervalTimer>,
    cron_smear: Duration,
}

impl Manager {
    pub fn new(guild_id: Id<GuildMarker>, storage: Db) -> Self {
        Self {
            storage,
            guild_id,
            loaded_intervals: HashMap::new(),
            pending: Vec::new(),
            cron_smear: Duration::milliseconds((guild_id.get() % 10000) as i64),
        }
    }

    pub fn next_action(&mut self) -> NextAction {
        let next_time = if let Some(next) = self.next_event_time() {
            next
        } else {
            return NextAction::None;
        };

        let now = Utc::now();
        if now > next_time {
            // we need to trigger some timers
            NextAction::Run
        } else {
            NextAction::Wait(next_time)
        }
    }

    pub async fn trigger_timers(&mut self) -> Vec<IntervalTimer> {
        let next_time = if let Some(next) = self.next_event_time() {
            next
        } else {
            return Vec::new();
        };

        let now = Utc::now();
        if now > next_time {
            // we need to trigger some timers
            let triggered = self.get_triggered_timers(now);
            for t in &triggered {
                self.pending.push(t.inner.clone())
            }
            triggered.into_iter().map(|v| v.inner).collect()
        } else {
            Vec::new()
        }
    }

    pub async fn timer_ack(&mut self, timer_id: &TimerId) {
        if let Some(index) = self
            .pending
            .iter()
            .position(|v| timer_id.identifies_timer(v))
        {
            let timer = self.pending.swap_remove(index);
            self.update_last_run(Utc::now(), timer).await;
        }
    }

    pub async fn script_started(&mut self, timers: Vec<IntervalTimerContrib>) {
        if timers.is_empty() {
            return;
        }

        info!("initializing {} timers", timers.len());
        let all_guild_timers = self
            .storage
            .get_all_guild_interval_timers(self.guild_id)
            .await
            .unwrap();

        for script_timer in timers {
            let db_timer = all_guild_timers
                .iter()
                .find(|v| v.name == script_timer.name && v.plugin_id == script_timer.plugin_id);

            self.init_timer(script_timer, db_timer).await.unwrap();
        }
    }

    pub fn clear_loaded_timers(&mut self) {
        self.loaded_intervals.clear();
    }
    pub fn clear_pending_acks(&mut self) {
        self.pending.clear();
    }

    pub fn remove_pending(&mut self, id: TimerId) {
        if let Some(index) = self.pending.iter().position(|v| id.identifies_timer(v)) {
            self.pending.swap_remove(index);
        }
    }

    fn get_triggered_timers(&self, now: DateTime<Utc>) -> Vec<WrappedIntervalTimer> {
        self.loaded_intervals
            .iter()
            .filter(|(name, _)| {
                !self
                    .pending
                    .iter()
                    .any(|pending| name.identifies_timer(pending))
            })
            .filter(|(_, t)| t.is_triggered(now))
            .map(|(_, t)| t.clone())
            .collect::<Vec<_>>()
    }

    async fn update_last_run(&mut self, t: DateTime<Utc>, triggered_timer: IntervalTimer) {
        let mut triggered_inner_clone = triggered_timer.clone();
        triggered_inner_clone.last_run = t;

        if let Err(err) = self
            .storage
            .update_interval_timer(self.guild_id, triggered_inner_clone)
            .await
        {
            tracing::error!(%err, "failed updating timer")
        };

        // update next time if the timer is loaded
        let Some(loaded_timer) = self.loaded_intervals.get_mut(&TimerId(
            triggered_timer.plugin_id,
            triggered_timer.name.clone(),
        )) else {
            // timer is not loaded, we have already updated the last run time so we don't need to do more
            return;
        };

        loaded_timer.inner.last_run = t;
        if let Some(next) = loaded_timer.parsed_type.next_run_time(t) {
            loaded_timer.next_run = next;
        } else {
            // TODO: proper handling of unknown next time, this could happen with invalid cron times for example
            loaded_timer.next_run = t.add(Duration::days(100 * 365));
        }
    }

    async fn init_timer(
        &mut self,
        script_timer: IntervalTimerContrib,
        db_timer: Option<&IntervalTimer>,
    ) -> Result<(), anyhow::Error> {
        let last_run = db_timer
            .map(|v| v.last_run)
            .unwrap_or_else(chrono::Utc::now);

        let timer = self
            .storage
            .update_interval_timer(
                self.guild_id,
                IntervalTimer {
                    last_run,
                    interval: script_timer.interval,
                    name: script_timer.name,
                    plugin_id: script_timer.plugin_id,
                },
            )
            .await?;

        match wrap_timer(timer) {
            Ok(wrapped) => {
                self.loaded_intervals.insert(
                    TimerId(wrapped.inner.plugin_id, wrapped.inner.name.clone()),
                    wrapped,
                );
            }
            Err(err) => tracing::error!(?err, "failed wrapping timer"),
        };

        Ok(())
    }

    pub fn next_event_time(&self) -> Option<DateTime<Utc>> {
        let lowest_interval = self
            .loaded_intervals
            .iter()
            .filter(|(name, _)| {
                // filter timers already triggered that were waiting to hear back from being run
                !self
                    .pending
                    .iter()
                    .any(|pending| name.identifies_timer(pending))
            })
            .min_by(|(_, a), (_, b)| a.next_run.cmp(&b.next_run));

        lowest_interval.map(|(_, v)| v.next_run_with_cron_smear(self.cron_smear))
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
            let parsed =
                cron::Schedule::from_str(format!("0 {c}").as_str()).map_err(Error::Cron)?;
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

    fn next_run_with_cron_smear(&self, cron_smear: Duration) -> DateTime<Utc> {
        match self.parsed_type {
            ParsedIntervalType::Minutes(_) => self.next_run,
            ParsedIntervalType::Cron(_, _) => self.next_run + cron_smear,
        }
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

pub type NextAction = crate::guild_handler::NextTimerAction;
