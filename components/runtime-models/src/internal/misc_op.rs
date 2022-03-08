use serde::Deserialize;
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateBanFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct CreateBanFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_log_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_message_days: Option<u32>,
}
