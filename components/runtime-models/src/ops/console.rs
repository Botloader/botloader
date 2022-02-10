use serde::Deserialize;
use ts_rs::TS;
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/ConsoleLogMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct LogMessage {
    #[serde(default)]
    #[ts(optional)]
    pub file_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub line_number: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub col_number: Option<u32>,

    pub message: String,
}
