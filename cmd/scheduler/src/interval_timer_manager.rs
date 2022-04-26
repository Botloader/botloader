use std::{collections::HashMap, ops::Add, str::FromStr, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use stores::{
    config::IntervalTimerContrib,
    timers::{IntervalTimer, IntervalType},
};
use tracing::info;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::scheduler;

#[derive(Debug)]
pub enum Error {
    Cron(cron::error::Error),
    NoNextTime,
}

pub struct Manager {
    storage: Arc<dyn scheduler::Store>,
    guild_id: Id<GuildMarker>,
    loaded_intervals: HashMap<String, WrappedIntervalTimer>,
    pending: Vec<String>,
}

impl Manager {
    pub fn new(guild_id: Id<GuildMarker>, storage: Arc<dyn scheduler::Store>) -> Self {
        Self {
            storage,
            guild_id,
            loaded_intervals: HashMap::new(),
            pending: Vec::new(),
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
                self.pending.push(t.inner.name.clone())
            }
            triggered.into_iter().map(|v| v.inner).collect()
        } else {
            Vec::new()
        }
    }

    pub async fn timer_ack(&mut self, timer: String) {
        if let Some(index) =
            self.pending
                .iter()
                .enumerate()
                .find_map(|(i, v)| if *v == timer { Some(i) } else { None })
        {
            self.pending.swap_remove(index);
            self.update_next_run(Utc::now(), timer).await;
        }
    }

    // pub async fn timer_ack_failed(&mut self, timer: String) {
    //     if let Some(index) =
    //         self.pending
    //             .iter()
    //             .enumerate()
    //             .find_map(|(i, v)| if *v == timer { Some(i) } else { None })
    //     {
    //         self.pending.swap_remove(index);
    //     }
    // }

    pub async fn script_started(&mut self, timers: Vec<IntervalTimerContrib>) {
        if timers.is_empty() {
            return;
        }

        info!("initialzing {} timers", timers.len());
        let all_guild_timers = self
            .storage
            .get_all_interval_timers(self.guild_id)
            .await
            .unwrap();

        for script_timer in timers {
            let db_timer = all_guild_timers
                .iter()
                .find(|v| v.name == script_timer.name);

            self.init_timer(script_timer, db_timer).await.unwrap();
        }
    }

    pub fn clear_loaded_timers(&mut self) {
        self.loaded_intervals.clear();
    }
    pub fn clear_pending_acks(&mut self) {
        self.pending.clear();
    }

    fn get_triggered_timers(&self, now: DateTime<Utc>) -> Vec<WrappedIntervalTimer> {
        self.loaded_intervals
            .iter()
            .filter(|(name, _)| !self.pending.contains(name))
            .filter(|(_, t)| t.is_triggered(now))
            .map(|(_, t)| t.clone())
            .collect::<Vec<_>>()
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

        Ok(())
    }

    pub fn next_event_time(&self) -> Option<DateTime<Utc>> {
        let lowest_interval = self
            .loaded_intervals
            .iter()
            .filter(|(name, _)| !self.pending.contains(name))
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
            let parsed =
                cron::Schedule::from_str(format!("0 {}", c).as_str()).map_err(Error::Cron)?;
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

pub type NextAction = crate::guild_handler::NextTimerAction;
