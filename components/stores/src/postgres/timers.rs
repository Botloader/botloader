use std::convert::TryFrom;

use crate::timers::{
    IntervalTimer, IntervalType, ScheduledTask, TimerStoreError, TimerStoreResult,
};

use super::Postgres;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use twilight_model::id::GuildId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("minute and cron interval both not set")]
    NoMinutesOrCronInterval,
}

impl From<Error> for TimerStoreError {
    fn from(v: Error) -> Self {
        Self::Other(Box::new(v))
    }
}

impl From<sqlx::Error> for TimerStoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

#[async_trait]
impl crate::timers::TimerStore for Postgres {
    async fn get_all_interval_timers(
        &self,
        guild_id: GuildId,
    ) -> TimerStoreResult<Vec<IntervalTimer>> {
        let res = sqlx::query_as!(
            DbIntervalTimer,
            "SELECT guild_id, script_id, timer_name, interval_minutes, interval_cron, \
             last_run_at, created_at, updated_at
            FROM interval_timers WHERE guild_id=$1;",
            guild_id.get() as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res
            .into_iter()
            .filter_map(|v| IntervalTimer::try_from(v).ok())
            .collect())
    }

    async fn update_interval_timer(
        &self,
        guild_id: GuildId,
        timer: IntervalTimer,
    ) -> TimerStoreResult<IntervalTimer> {
        let (interval_minutes, interval_cron) = match timer.interval {
            IntervalType::Minutes(m) => (Some(m as i32), None),
            IntervalType::Cron(c) => (None, Some(c)),
        };

        let res = sqlx::query_as!(
            DbIntervalTimer,
            "
            INSERT INTO interval_timers (guild_id, script_id, timer_name, interval_minutes, \
             interval_cron, last_run_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, now(), now())
            ON CONFLICT (guild_id, script_id, timer_name)
            DO UPDATE SET
            interval_minutes = $4,
            interval_cron = $5,
            last_run_at = $6,
            updated_at = now()
            RETURNING guild_id, script_id, timer_name, interval_minutes, interval_cron, \
             last_run_at, created_at, updated_at;
             ",
            guild_id.get() as i64,
            0,
            timer.name,
            interval_minutes,
            interval_cron,
            timer.last_run,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(IntervalTimer::try_from(res)?)
    }

    async fn del_interval_timer(
        &self,
        guild_id: GuildId,
        script_id: u64,
        timer_name: String,
    ) -> TimerStoreResult<bool> {
        let res = sqlx::query!(
            "DELETE FROM interval_timers WHERE guild_id=$1 AND script_id=$2 AND timer_name=$3",
            guild_id.get() as i64,
            script_id as i64,
            timer_name
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }

    async fn create_task(
        &self,
        guild_id: GuildId,
        name: String,
        unique_key: Option<String>,
        data: serde_json::Value,
        at: DateTime<Utc>,
    ) -> TimerStoreResult<ScheduledTask> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "INSERT INTO scheduled_tasks (guild_id, name, unique_key, value, exec_at) VALUES($1, \
             $2, $3, $4, $5)
            ON CONFLICT (guild_id, name, unique_key) WHERE unique_key IS NOT NULL DO UPDATE SET
            value = excluded.value,
            exec_at = excluded.exec_at
            RETURNING id, guild_id, name, unique_key, value, exec_at",
            guild_id.get() as i64,
            name,
            unique_key,
            data,
            at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn get_task_by_id(
        &self,
        guild_id: GuildId,
        id: u64,
    ) -> TimerStoreResult<Option<ScheduledTask>> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, name, unique_key, value, exec_at FROM scheduled_tasks WHERE \
             guild_id = $1 AND id = $2",
            guild_id.get() as i64,
            id as i64,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into))
    }
    async fn get_task_by_key(
        &self,
        guild_id: GuildId,
        name: String,
        key: String,
    ) -> TimerStoreResult<Option<ScheduledTask>> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, name, unique_key, value, exec_at FROM scheduled_tasks WHERE \
             guild_id = $1 AND name = $2 AND unique_key = $3",
            guild_id.get() as i64,
            name,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into))
    }

    async fn get_tasks(
        &self,
        guild_id: GuildId,
        name: Option<String>,
        id_after: u64,
        limit: usize,
    ) -> TimerStoreResult<Vec<ScheduledTask>> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, name, unique_key, value, exec_at FROM scheduled_tasks WHERE \
             guild_id = $1 AND (name = $2 OR $2 IS NULL) AND id > $3 ORDER BY ID ASC LIMIT $4",
            guild_id.get() as i64,
            name,
            id_after as i64,
            limit as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    /// Delete a task by the global unique ID
    async fn del_task_by_id(&self, guild_id: GuildId, id: u64) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1 AND id = $2",
            guild_id.get() as i64,
            id as i64,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    /// Delete one or more tasks by their (guild_id, name) unique key
    /// (does nothing to key = null tasks)
    async fn del_task_by_key(
        &self,
        guild_id: GuildId,
        name: String,
        key: String,
    ) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1 AND name = $2 AND unique_key = $3",
            guild_id.get() as i64,
            name,
            key
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    /// Delete all tasks on a guild, optionally filtered by name
    async fn del_all_tasks(
        &self,
        guild_id: GuildId,
        name: Option<String>,
    ) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1 AND (name = $2 OR $2 IS NULL )",
            guild_id.get() as i64,
            name,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    async fn get_next_task_time(
        &self,
        guild_id: GuildId,
        ignore_ids: &[u64],
        names: &[String],
    ) -> TimerStoreResult<Option<DateTime<Utc>>> {
        let res = sqlx::query!(
            "SELECT exec_at FROM scheduled_tasks WHERE guild_id = $1 AND name = ANY($2::TEXT[]) \
             AND (NOT id = ANY ($3::BIGINT[])) ORDER BY exec_at ASC LIMIT 1",
            guild_id.get() as i64,
            names,
            &ignore_ids.iter().map(|v| *v as i64).collect::<Vec<_>>(),
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(|v| v.exec_at))
    }
    async fn get_triggered_tasks(
        &self,
        guild_id: GuildId,
        t: DateTime<Utc>,
        ignore_ids: &[u64],
        names: &[String],
    ) -> TimerStoreResult<Vec<ScheduledTask>> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, name, unique_key, value, exec_at FROM scheduled_tasks WHERE \
             guild_id = $1 AND exec_at < $2 AND name = ANY($3::TEXT[]) AND (NOT id = ANY \
             ($4::BIGINT[]))",
            guild_id.get() as i64,
            t,
            names,
            &ignore_ids.iter().map(|v| *v as i64).collect::<Vec<_>>(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    async fn get_task_count(&self, guild_id: GuildId) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "SELECT COUNT(*) FROM scheduled_tasks WHERE guild_id = $1;",
            guild_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.count.unwrap_or_default() as u64)
    }

    async fn delete_guild_timer_data(&self, id: GuildId) -> TimerStoreResult<()> {
        sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1;",
            id.get() as i64
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "DELETE FROM interval_timers WHERE guild_id = $1;",
            id.get() as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct DbIntervalTimer {
    #[allow(dead_code)]
    guild_id: i64,
    #[allow(dead_code)]
    script_id: i64,
    timer_name: String,
    interval_minutes: Option<i32>,
    interval_cron: Option<String>,
    last_run_at: DateTime<Utc>,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    #[allow(dead_code)]
    updated_at: DateTime<Utc>,
}

impl TryFrom<DbIntervalTimer> for IntervalTimer {
    type Error = Error;

    fn try_from(value: DbIntervalTimer) -> Result<Self, Self::Error> {
        let interval_type = if let Some(mins) = value.interval_minutes {
            IntervalType::Minutes(mins as u64)
        } else if let Some(cron_text) = value.interval_cron {
            IntervalType::Cron(cron_text)
        } else {
            return Err(Error::NoMinutesOrCronInterval);
        };

        Ok(Self {
            name: value.timer_name,
            last_run: value.last_run_at,
            interval: interval_type,
        })
    }
}

struct DbScheduledTask {
    id: i64,
    #[allow(dead_code)]
    guild_id: i64,
    name: String,
    unique_key: Option<String>,
    value: serde_json::Value,
    exec_at: DateTime<Utc>,
}

impl From<DbScheduledTask> for ScheduledTask {
    fn from(v: DbScheduledTask) -> Self {
        Self {
            id: v.id as u64,
            name: v.name,
            unique_key: v.unique_key,
            data: v.value,
            execute_at: v.exec_at,
        }
    }
}
