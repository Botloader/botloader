use std::{cell::RefCell, rc::Rc};

use chrono::TimeZone;
use deno_core::{op2, OpState};
use runtime_models::{
    internal::tasks::{CreateScheduledTask, GetGuildTasksFilter, ScheduledTask},
    util::PluginId,
};
use vm::AnyError;

use crate::{get_rt_ctx, limits::RateLimiters, RuntimeEvent};

deno_core::extension!(
    bl_tasks,
    ops = [
        op_bl_schedule_task,
        op_bl_del_task,
        op_bl_del_task_by_key,
        op_bl_del_all_tasks,
        op_bl_get_task,
        op_bl_get_task_by_key,
        op_bl_get_all_tasks,
    ],
);

#[op2(async)]
#[serde]
async fn op_bl_schedule_task(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: CreateScheduledTask,
) -> Result<ScheduledTask, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    RateLimiters::task_ops(&state).await;

    let seconds = (opts.execute_at.0 as f64 / 1000f64).floor() as i64;
    let millis = opts.execute_at.0 as i64 - (seconds * 1000);
    let t = chrono::Utc
        .timestamp_opt(seconds, millis as u32 * 1_000_000)
        .unwrap();

    let data_serialized = serde_json::to_string(&opts.data)?;
    let limit_data_len = crate::limits::tasks_data_size(&state);
    if data_serialized.len() as u64 > limit_data_len {
        return Err(anyhow::anyhow!(
            "data cannot be over {limit_data_len}bytes on your guild's plan"
        ));
    }

    // TODO: make a more efficient check
    let current = rt_ctx.db.get_task_count(rt_ctx.guild_id).await?;
    let limit_num_tasks = crate::limits::tasks_scheduled_count(&state);
    if current > limit_num_tasks {
        return Err(anyhow::anyhow!(
            "max {limit_num_tasks} can be scheduled on this guild's plan"
        ));
    }

    let res = rt_ctx
        .db
        .create_task(
            rt_ctx.guild_id,
            opts.plugin_id.map(Into::into),
            opts.namespace,
            opts.unique_key,
            opts.data,
            t,
        )
        .await?
        .into();

    let _ = rt_ctx.event_tx.send(RuntimeEvent::NewTaskScheduled);

    Ok(res)
}

#[op2(async)]
async fn op_bl_del_task(
    state: Rc<RefCell<OpState>>,
    #[number] task_id: u64,
) -> Result<bool, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    RateLimiters::task_ops(&state).await;

    let del = rt_ctx.db.del_task_by_id(rt_ctx.guild_id, task_id).await?;
    Ok(del > 0)
}

#[op2(async)]
async fn op_bl_del_task_by_key(
    state: Rc<RefCell<OpState>>,
    #[serde] plugin_id: Option<PluginId>,
    #[string] name: String,
    #[string] key: String,
) -> Result<bool, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    RateLimiters::task_ops(&state).await;

    let del = rt_ctx
        .db
        .del_task_by_key(rt_ctx.guild_id, plugin_id.map(Into::into), name, key)
        .await?;

    Ok(del > 0)
}

#[op2(async)]
#[number]
async fn op_bl_del_all_tasks(
    state: Rc<RefCell<OpState>>,
    #[serde] plugin_id: Option<PluginId>,
    #[string] name: String,
) -> Result<u64, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    RateLimiters::task_ops(&state).await;

    let del = rt_ctx
        .db
        .del_all_tasks(rt_ctx.guild_id, plugin_id.map(Into::into), Some(name))
        .await?;
    Ok(del)
}

#[op2(async)]
#[serde]
async fn op_bl_get_task(
    state: Rc<RefCell<OpState>>,
    #[number] id: u64,
) -> Result<Option<ScheduledTask>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    RateLimiters::task_ops(&state).await;

    Ok(rt_ctx
        .db
        .get_task_by_id(rt_ctx.guild_id, id)
        .await?
        .map(Into::into))
}

#[op2(async)]
#[serde]
async fn op_bl_get_task_by_key(
    state: Rc<RefCell<OpState>>,
    #[serde] plugin_id: Option<PluginId>,
    #[string] name: String,
    #[string] key: String,
) -> Result<Option<ScheduledTask>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    RateLimiters::task_ops(&state).await;

    Ok(rt_ctx
        .db
        .get_task_by_key(rt_ctx.guild_id, plugin_id.map(Into::into), name, key)
        .await?
        .map(Into::into))
}

#[op2(async)]
#[serde]
async fn op_bl_get_all_tasks(
    state: Rc<RefCell<OpState>>,
    #[serde] filter: GetGuildTasksFilter,
    #[number] after_id: u64,
) -> Result<Vec<ScheduledTask>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    RateLimiters::task_ops(&state).await;

    Ok(rt_ctx
        .db
        .get_guild_tasks(rt_ctx.guild_id, filter, after_id, 25)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}
