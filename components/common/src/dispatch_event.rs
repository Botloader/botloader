use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum EventSource {
    Discord,
    Timer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VmDispatchEvent {
    pub name: String,
    pub seq: u64,
    pub value: serde_json::Value,
    pub source: EventSource,
    pub source_timestamp: chrono::DateTime<chrono::Utc>,
}
