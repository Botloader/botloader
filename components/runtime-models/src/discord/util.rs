use serde::Deserialize;
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/AuditLogExtras.ts")]
pub struct AuditLogExtras {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_log_reason: Option<String>,
}
