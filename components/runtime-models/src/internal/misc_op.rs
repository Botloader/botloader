use serde::Deserialize;
use ts_rs::TS;

use crate::discord::message::SendEmoji;

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

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/GetReactions.ts")]
#[serde(rename_all = "camelCase")]
pub struct GetReactionsFields {
    pub emoji: SendEmoji,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
