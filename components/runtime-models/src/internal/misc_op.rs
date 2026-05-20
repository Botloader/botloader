use serde::Deserialize;
use ts_rs::TS;

use crate::discord::message::SendEmoji;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateBanFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct CreateBanFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub audit_log_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delete_message_days: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/GetReactions.ts")]
#[serde(rename_all = "camelCase")]
pub struct GetReactionsFields {
    pub emoji: SendEmoji,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub limit: Option<u32>,
}
