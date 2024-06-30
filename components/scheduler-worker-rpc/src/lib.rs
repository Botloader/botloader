use std::collections::HashMap;

use common::dispatch_event::VmDispatchEvent;
use runtime_models::internal::script::ScriptMeta;
use serde::{Deserialize, Serialize};
use stores::config::{PremiumSlotTier, Script};
use twilight_model::id::{marker::GuildMarker, Id};
use vm::vm::ShutdownReason;

#[derive(Deserialize, Serialize)]
pub enum SchedulerMessage {
    Dispatch(VmDispatchEvent),
    /// stops the current vm and creates a new one to run the provided scripts
    CreateScriptsVm(CreateScriptsVmReq),
    Complete,
    Shutdown,
}

impl SchedulerMessage {
    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        match self {
            SchedulerMessage::Dispatch(_) => None,
            SchedulerMessage::CreateScriptsVm(v) => Some(v.guild_id),
            SchedulerMessage::Complete => None,
            SchedulerMessage::Shutdown => None,
        }
    }

    pub fn span_name(&self) -> &'static str {
        match self {
            SchedulerMessage::Dispatch(_) => "SchedulerMessage::Dispatch",
            SchedulerMessage::CreateScriptsVm(_) => "SchedulerMessage::CreateScriptsVm",
            SchedulerMessage::Complete => "SchedulerMessage::Complete",
            SchedulerMessage::Shutdown => "SchedulerMessage::Shutdown",
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateScriptsVmReq {
    pub seq: u64,
    pub session_id: u64,
    pub premium_tier: Option<PremiumSlotTier>,
    pub guild_id: Id<GuildMarker>,
    pub scripts: Vec<Script>,
}

#[derive(Deserialize, Serialize)]
pub struct VmSessionShutdownEvent {
    pub vm_session_id: u64,
    pub guild_id: Id<GuildMarker>,
    pub reason: Option<ShutdownReason>,
}

#[derive(Deserialize, Serialize)]
pub enum WorkerMessage {
    Ack(u64),
    Shutdown(VmSessionShutdownEvent),
    ScriptStarted(ScriptMeta),
    ScriptsInit,
    NonePending,
    TaskScheduled,
    GuildLog(guild_logger::LogEntry),
    Hello(u64),
    Metric(String, MetricEvent, HashMap<String, String>),
}

impl WorkerMessage {
    pub fn name(&self) -> &'static str {
        match self {
            WorkerMessage::Ack(_) => "Ack",
            WorkerMessage::Shutdown(_) => "Shutdown",
            WorkerMessage::ScriptStarted(_) => "ScriptStarted",
            WorkerMessage::ScriptsInit => "ScriptsInit",
            WorkerMessage::NonePending => "NonePending",
            WorkerMessage::TaskScheduled => "TaskScheduled",
            WorkerMessage::GuildLog(_) => "GuildLog",
            WorkerMessage::Hello(_) => "Hello",
            WorkerMessage::Metric(_, _, _) => "Metric",
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MetricEvent {
    Gauge(GaugeEvent),
    Counter(CounterEvent),
    Histogram(f64),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum GaugeEvent {
    Set(f64),
    Incr(f64),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum CounterEvent {
    Incr(u64),
    Absolute(u64),
}
