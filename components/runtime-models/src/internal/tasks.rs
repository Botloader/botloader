use crate::util::NotBigU64;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateScheduledTask.ts")]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduledTask {
    pub namespace: String,

    #[serde(default)]
    #[ts(optional)]
    pub unique_key: Option<String>,

    #[ts(type = "any")]
    pub data: serde_json::Value,
    pub execute_at: NotBigU64,
}
