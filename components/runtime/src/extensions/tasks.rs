use std::{cell::RefCell, rc::Rc};

use chrono::TimeZone;
use deno_core::{op_async, Extension, OpState};
use runtime_models::{events::task::ScheduledTask, ops::tasks::CreateScheduledTask};
use vm::AnyError;

use crate::{wrap_bl_op_async, RuntimeContext, RuntimeEvent};

pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![
            // botloader stuff
            (
                "op_bl_schedule_task",
                op_async(wrap_bl_op_async(op_bl_schedule_task)),
            ),
            ("op_bl_del_task", op_async(wrap_bl_op_async(op_bl_del_task))),
            (
                "op_bl_del_task_by_key",
                op_async(wrap_bl_op_async(op_bl_del_task_by_key)),
            ),
            (
                "op_bl_del_all_tasks",
                op_async(wrap_bl_op_async(op_bl_del_all_tasks)),
            ),
            ("op_bl_get_task", op_async(wrap_bl_op_async(op_bl_get_task))),
            (
                "op_bl_get_task_by_key",
                op_async(wrap_bl_op_async(op_bl_get_task_by_key)),
            ),
            (
                "op_bl_get_all_tasks",
                op_async(wrap_bl_op_async(op_bl_get_all_tasks)),
            ),
        ])
        .build()
}

async fn op_bl_schedule_task(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    opts: CreateScheduledTask,
    _: (),
) -> Result<ScheduledTask, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    let seconds = (opts.execute_at.0 as f64 / 1000f64).floor() as i64;
    let millis = opts.execute_at.0 as i64 - (seconds * 1000);
    let t = chrono::Utc.timestamp(seconds, millis as u32 * 1_000_000);

    let data_serialized = serde_json::to_string(&opts.data)?;
    if data_serialized.len() > 10_000 {
        return Err(anyhow::anyhow!("data cannot be over 10KB"));
    }

    // TODO: make a more efficient check
    let current = rt_ctx.timer_store.get_task_count(rt_ctx.guild_id).await?;
    if current > 100_000 {
        return Err(anyhow::anyhow!("over 100k tasks scheduled"));
    }

    let res = rt_ctx
        .timer_store
        .create_task(
            rt_ctx.guild_id,
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

async fn op_bl_del_task(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    task_id: u64,
    _: (),
) -> Result<bool, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    let del = rt_ctx
        .timer_store
        .del_task_by_id(rt_ctx.guild_id, task_id)
        .await?;
    Ok(del > 0)
}

async fn op_bl_del_task_by_key(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    name: String,
    key: String,
) -> Result<bool, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    let del = rt_ctx
        .timer_store
        .del_task_by_key(rt_ctx.guild_id, name, key)
        .await?;

    Ok(del > 0)
}

async fn op_bl_del_all_tasks(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    name: String,
    _: (),
) -> Result<u64, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    let del = rt_ctx
        .timer_store
        .del_all_tasks(rt_ctx.guild_id, Some(name))
        .await?;
    Ok(del)
}

async fn op_bl_get_task(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    id: u64,
    _: (),
) -> Result<Option<ScheduledTask>, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    Ok(rt_ctx
        .timer_store
        .get_task_by_id(rt_ctx.guild_id, id)
        .await?
        .map(Into::into))
}

async fn op_bl_get_task_by_key(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    name: String,
    key: String,
) -> Result<Option<ScheduledTask>, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    Ok(rt_ctx
        .timer_store
        .get_task_by_key(rt_ctx.guild_id, name, key)
        .await?
        .map(Into::into))
}

async fn op_bl_get_all_tasks(
    state: Rc<RefCell<OpState>>,
    rt_ctx: RuntimeContext,
    name: Option<String>,
    after_id: u64,
) -> Result<Vec<ScheduledTask>, AnyError> {
    crate::RateLimiters::ops_until_ready(state.clone(), crate::RatelimiterType::Tasks).await;

    Ok(rt_ctx
        .timer_store
        .get_tasks(rt_ctx.guild_id, name, after_id, 25)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}
