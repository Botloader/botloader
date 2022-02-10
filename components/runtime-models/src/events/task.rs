use serde::Serialize;
use ts_rs::TS;

use crate::util::NotBigU64;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/ScheduledTask.ts")]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: NotBigU64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub execute_at: NotBigU64,

    #[ts(type = "unknown")]
    pub data: serde_json::Value,
}

impl From<stores::timers::ScheduledTask> for ScheduledTask {
    fn from(v: stores::timers::ScheduledTask) -> Self {
        Self {
            id: NotBigU64(v.id),
            namespace: v.name,
            key: v.unique_key,
            execute_at: NotBigU64(v.execute_at.timestamp_millis() as u64),
            data: v.data,
        }
    }
}
