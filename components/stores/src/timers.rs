use std::convert::TryFrom;

use crate::Db;

use chrono::{DateTime, Utc};
use runtime_models::{
    internal::{
        script::TaskBucketId,
        tasks::{GetGuildTasksFilter, ScopeSelector},
    },
    util::{NotBigU64, PluginId},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twilight_model::id::{marker::GuildMarker, Id};

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

impl Db {
    pub async fn get_all_guild_interval_timers(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> TimerStoreResult<Vec<IntervalTimer>> {
        let res = sqlx::query_as!(
            DbIntervalTimer,
            "SELECT guild_id, plugin_id, timer_name, interval_minutes, interval_cron, \
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

    pub async fn update_interval_timer(
        &self,
        guild_id: Id<GuildMarker>,
        timer: IntervalTimer,
    ) -> TimerStoreResult<IntervalTimer> {
        let (interval_minutes, interval_cron) = match timer.interval {
            IntervalType::Minutes(m) => (Some(m as i32), None),
            IntervalType::Cron(c) => (None, Some(c)),
        };

        let res = sqlx::query_as!(
            DbIntervalTimer,
            "
            INSERT INTO interval_timers (guild_id, plugin_id, timer_name, interval_minutes, \
             interval_cron, last_run_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, now(), now())
            ON CONFLICT (guild_id, plugin_id, timer_name)
            DO UPDATE SET
            interval_minutes = $4,
            interval_cron = $5,
            last_run_at = $6,
            updated_at = now()
            RETURNING guild_id, plugin_id, timer_name, interval_minutes, interval_cron, \
             last_run_at, created_at, updated_at;
             ",
            guild_id.get() as i64,
            timer.plugin_id.unwrap_or(0) as i64,
            timer.name,
            interval_minutes,
            interval_cron,
            timer.last_run,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(IntervalTimer::try_from(res)?)
    }

    // async fn del_interval_timer(
    //     &self,
    //     guild_id: Id<GuildMarker>,
    //     plugin_id: Option<u64>,
    //     timer_name: String,
    // ) -> TimerStoreResult<bool> {
    //     let res = sqlx::query!(
    //         "DELETE FROM interval_timers WHERE guild_id=$1 AND plugin_id=$2 AND timer_name=$3",
    //         guild_id.get() as i64,
    //         plugin_id.unwrap_or(0) as i64,
    //         timer_name
    //     )
    //     .execute(&self.pool)
    //     .await?;

    //     Ok(res.rows_affected() > 0)
    // }

    pub async fn create_task(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        name: String,
        unique_key: Option<String>,
        data: serde_json::Value,
        at: DateTime<Utc>,
    ) -> TimerStoreResult<ScheduledTask> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "INSERT INTO scheduled_tasks (guild_id, plugin_id, name, unique_key, value, exec_at) \
             VALUES($1, $2, $3, $4, $5, $6)
            ON CONFLICT (guild_id, plugin_id, name, unique_key) WHERE unique_key IS NOT NULL DO \
             UPDATE SET
            value = excluded.value,
            exec_at = excluded.exec_at
            RETURNING id, guild_id, plugin_id, name, unique_key, value, exec_at",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            name,
            unique_key,
            data,
            at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    pub async fn get_task_by_id(
        &self,
        guild_id: Id<GuildMarker>,
        id: u64,
    ) -> TimerStoreResult<Option<ScheduledTask>> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, plugin_id, name, unique_key, value, exec_at FROM \
             scheduled_tasks WHERE guild_id = $1 AND id = $2",
            guild_id.get() as i64,
            id as i64,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into))
    }
    pub async fn get_task_by_key(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        name: String,
        key: String,
    ) -> TimerStoreResult<Option<ScheduledTask>> {
        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, plugin_id, name, unique_key, value, exec_at FROM \
             scheduled_tasks WHERE guild_id = $1 AND plugin_id = $2 AND name = $3 AND unique_key \
             = $4",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            name,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(Into::into))
    }

    pub async fn get_guild_tasks(
        &self,
        guild_id: Id<GuildMarker>,
        filter: GetGuildTasksFilter,
        id_after: u64,
        limit: usize,
    ) -> TimerStoreResult<Vec<ScheduledTask>> {
        let filter_plugin = match filter.scope {
            ScopeSelector::All => None,
            ScopeSelector::Guild => Some(0),
            ScopeSelector::Plugin { plugin_id } => Some(plugin_id.0),
        };

        let res = if let Some(plugin_id) = filter_plugin {
            sqlx::query_as!(
                DbScheduledTask,
                "SELECT id, guild_id, plugin_id, name, unique_key, value, exec_at FROM \
                 scheduled_tasks WHERE guild_id = $1 AND plugin_id = $2 AND (name = $3 OR $3 IS \
                 NULL) AND id > $4 ORDER BY ID ASC LIMIT $5",
                guild_id.get() as i64,
                plugin_id as i64,
                filter.namespace,
                id_after as i64,
                limit as i64,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                DbScheduledTask,
                "SELECT id, guild_id, plugin_id, name, unique_key, value, exec_at FROM \
                 scheduled_tasks WHERE guild_id = $1 AND (name = $2 OR $2 IS NULL) AND id > $3 \
                 ORDER BY ID ASC LIMIT $4",
                guild_id.get() as i64,
                filter.namespace,
                id_after as i64,
                limit as i64,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(res.into_iter().map(Into::into).collect())
    }

    /// Delete a task by the global unique ID
    pub async fn del_task_by_id(
        &self,
        guild_id: Id<GuildMarker>,
        id: u64,
    ) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1 AND id = $2",
            guild_id.get() as i64,
            id as i64,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn del_task_by_key(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        name: String,
        key: String,
    ) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1 AND plugin_id = $2 AND name = $3 AND \
             unique_key = $4",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            name,
            key
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    /// Delete all tasks on a guild, optionally filtered by name
    pub async fn del_all_tasks(
        &self,
        guild_id: Id<GuildMarker>,
        plugin_id: Option<u64>,
        name: Option<String>,
    ) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "DELETE FROM scheduled_tasks WHERE guild_id = $1 AND plugin_id = $2 AND (name = $3 OR \
             $3 IS NULL )",
            guild_id.get() as i64,
            plugin_id.unwrap_or(0) as i64,
            name,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_next_task_time(
        &self,
        guild_id: Id<GuildMarker>,
        ignore_ids: &[u64],
        names: &[TaskBucketId],
    ) -> TimerStoreResult<Option<DateTime<Utc>>> {
        // This was a pretty fun rabbit hole to go down
        // The problem is that postgres' multidimensional array support is trash.
        // If i were to ask you what would the result of:
        // ARRAY[1,2] = ANY (ARRAY[ARRAY[1,2], ARRAY[3,4]])
        // You would be a fool to say "true", this is actually invalid because ANY just does not care about dimensions.
        //
        // in fact, as i discovered, THERE IS NO WAY TO CHECK IF AN ARRAY IS CONTAINED IN A MULTI DIMENSIONAL ARRAY.
        //
        // So to work around this shitty flaw, just concat the nested array to a string, shitty but works for now.
        let name_plugin_filter = names
            .iter()
            .map(|v| format!("{}_{}", v.plugin_id.unwrap_or(PluginId(0)).0, v.name))
            .collect::<Vec<_>>();

        let res = sqlx::query!(
            "SELECT exec_at FROM scheduled_tasks WHERE guild_id = $1 AND plugin_id || '_' || name \
             = ANY($2::text[]) AND (NOT id = ANY ($3::BIGINT[])) ORDER BY exec_at ASC LIMIT 1",
            guild_id.get() as i64,
            &name_plugin_filter,
            &ignore_ids.iter().map(|v| *v as i64).collect::<Vec<_>>(),
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(|v| v.exec_at))
    }

    pub async fn get_triggered_tasks(
        &self,
        guild_id: Id<GuildMarker>,
        t: DateTime<Utc>,
        ignore_ids: &[u64],
        names: &[TaskBucketId],
    ) -> TimerStoreResult<Vec<ScheduledTask>> {
        // Working around postgres limitation, see the comment in get_next_task_time for an explanation
        let name_plugin_filter = names
            .iter()
            .map(|v| format!("{}_{}", v.plugin_id.unwrap_or(PluginId(0)).0, v.name))
            .collect::<Vec<_>>();

        let res = sqlx::query_as!(
            DbScheduledTask,
            "SELECT id, guild_id, plugin_id, name, unique_key, value, exec_at FROM \
             scheduled_tasks WHERE guild_id = $1 AND exec_at < $2 AND plugin_id || '_' || name = \
             ANY($3::text[]) AND (NOT id = ANY ($4::BIGINT[]))",
            guild_id.get() as i64,
            t,
            &name_plugin_filter,
            &ignore_ids.iter().map(|v| *v as i64).collect::<Vec<_>>(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(Into::into).collect())
    }

    pub async fn get_task_count(&self, guild_id: Id<GuildMarker>) -> TimerStoreResult<u64> {
        let res = sqlx::query!(
            "SELECT COUNT(*) FROM scheduled_tasks WHERE guild_id = $1;",
            guild_id.get() as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.count.unwrap_or_default() as u64)
    }

    pub async fn delete_guild_timer_data(&self, id: Id<GuildMarker>) -> TimerStoreResult<()> {
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
    plugin_id: i64,
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
            plugin_id: (value.plugin_id > 0).then_some(value.plugin_id as u64),
        })
    }
}

struct DbScheduledTask {
    id: i64,
    #[allow(dead_code)]
    guild_id: i64,
    plugin_id: i64,
    name: String,
    unique_key: Option<String>,
    value: serde_json::Value,
    exec_at: DateTime<Utc>,
}

impl From<DbScheduledTask> for ScheduledTask {
    fn from(v: DbScheduledTask) -> Self {
        Self {
            id: v.id as u64,
            plugin_id: (v.plugin_id > 0).then_some(v.plugin_id as u64),
            name: v.name,
            unique_key: v.unique_key,
            data: v.value,
            execute_at: v.exec_at,
        }
    }
}

#[derive(Clone)]
pub struct IntervalTimer {
    pub name: String,
    pub interval: IntervalType,
    pub last_run: DateTime<Utc>,
    // pub script_id: u64,
    pub plugin_id: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum IntervalType {
    Minutes(u64),
    Cron(String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ScheduledTask {
    pub id: u64,
    pub name: String,
    pub plugin_id: Option<u64>,

    pub unique_key: Option<String>,

    pub data: serde_json::Value,
    pub execute_at: DateTime<Utc>,
}

impl From<ScheduledTask> for runtime_models::internal::tasks::ScheduledTask {
    fn from(v: ScheduledTask) -> Self {
        Self {
            id: NotBigU64(v.id),
            plugin_id: v.plugin_id.map(PluginId),
            namespace: v.name,
            key: v.unique_key,
            execute_at: NotBigU64(v.execute_at.timestamp_millis() as u64),
            data: v.data,
        }
    }
}

#[derive(Debug, Error)]
pub enum TimerStoreError {
    #[error("inner error occurred: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type TimerStoreResult<T> = Result<T, TimerStoreError>;
