use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/IntervalTimerEvent.ts")]
#[serde(rename_all = "camelCase")]
pub struct IntervalTimerEvent {
    pub name: String,
}
