use serde::Serialize;
use ts_rs::TS;

use crate::util::PluginId;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/IntervalTimerEvent.ts")]
#[serde(rename_all = "camelCase")]
pub struct IntervalTimerEvent {
    pub name: String,
    pub plugin_id: Option<PluginId>,
}
