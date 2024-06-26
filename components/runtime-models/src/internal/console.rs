use serde::Deserialize;
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/ConsoleLogMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLogMessage {
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

    pub level: ConsoleLogLevel,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/ConsoleLogLevel.ts")]
#[serde(rename_all = "camelCase")]
pub enum ConsoleLogLevel {
    Log,
    Warn,
    Error,
}
