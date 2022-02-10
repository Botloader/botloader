use serde::Serialize;
use ts_rs::TS;

use crate::discord::user::User;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/MemberRemove.ts")]
#[serde(rename_all = "camelCase")]
pub struct MemberRemove {
    pub guild_id: String,
    pub user: User,
}

impl From<twilight_model::gateway::payload::incoming::MemberRemove> for MemberRemove {
    fn from(v: twilight_model::gateway::payload::incoming::MemberRemove) -> Self {
        Self {
            guild_id: v.guild_id.to_string(),
            user: v.user.into(),
        }
    }
}
