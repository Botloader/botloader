use std::collections::HashMap;

use runtime_models::internal::script::ScriptMeta;
use serde::{Deserialize, Serialize};
use stores::config::Script;
use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Deserialize, Serialize)]
pub enum SchedulerMessage {
    Dispatch(VmDispatchEvent),
    UpdateRunState(u64, UpdateRunStateRequest),
    Shutdown,
}

#[derive(Deserialize, Serialize)]
pub struct VmDispatchEvent {
    pub name: String,
    pub seq: u64,
    pub value: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub enum WorkerMessage {
    Ack(u64),
    Shutdown(ShutdownReason),
    ScriptStarted(ScriptMeta),
    ScriptsInit,
    NonePending,
    TaskScheduled,
    GuildLog(guild_logger::LogEntry),
    Hello(u64),
    Metric(String, MetricEvent, HashMap<String, String>),
}

#[derive(Deserialize, Serialize)]
pub struct UpdateRunStateRequest {
    pub guild_id: Id<GuildMarker>,
    pub guild_scripts: RunStateChangeReq<Vec<Script>>,
    pub packs: RunStateChangeReq<()>,
}

#[derive(Deserialize, Serialize)]
pub enum RunStateChangeReq<T> {
    Keep,
    Stop,
    Start(T),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ShutdownReason {
    Runaway,
    OutOfMemory,
    Other,
    TooManyInvalidRequests,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MetricEvent {
    Gauge(GaugeEvent),
    Counter(CounterEvent),
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
