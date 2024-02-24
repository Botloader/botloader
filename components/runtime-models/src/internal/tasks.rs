use crate::util::{NotBigU64, PluginId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateScheduledTask.ts")]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduledTask {
    pub plugin_id: Option<PluginId>,

    pub namespace: String,

    #[serde(default)]
    #[ts(optional)]
    pub unique_key: Option<String>,

    #[ts(type = "any")]
    pub data: serde_json::Value,
    pub execute_at: NotBigU64,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/ScheduledTask.ts")]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: NotBigU64,
    pub namespace: String,
    pub plugin_id: Option<PluginId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub execute_at: NotBigU64,

    #[ts(type = "unknown")]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/GetGuildTasksFilter.ts")]
#[serde(rename_all = "camelCase")]
pub struct GetGuildTasksFilter {
    pub scope: ScopeSelector,
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/ScopeSelector.ts")]
#[serde(tag = "kind")]
pub enum ScopeSelector {
    All,
    Guild,
    Plugin { plugin_id: PluginId },
}
