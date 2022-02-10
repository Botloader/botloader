use serde::Serialize;
use ts_rs::TS;

use crate::{
    discord::channel::{ChannelType, ThreadMetadata},
    util::NotBigU64,
};
use twilight_model::application::interaction::application_command;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/InteractionChannel.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionPartialChannel {
    pub id: String,
    pub kind: ChannelType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub permissions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_metadata: Option<ThreadMetadata>,
}

impl From<application_command::InteractionChannel> for InteractionPartialChannel {
    fn from(v: application_command::InteractionChannel) -> Self {
        Self {
            id: v.id.to_string(),
            kind: v.kind.into(),
            name: v.name,
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            permissions: v.permissions.bits().to_string(),
            thread_metadata: v.thread_metadata.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/InteractionPartialMember.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionPartialMember {
    pub joined_at: NotBigU64,
    pub nick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<NotBigU64>,
    #[serde(default)]
    pub roles: Vec<String>,
}

impl From<application_command::InteractionMember> for InteractionPartialMember {
    fn from(v: application_command::InteractionMember) -> Self {
        Self {
            joined_at: NotBigU64(v.joined_at.as_micros() as u64 / 1000),
            nick: v.nick,
            premium_since: v
                .premium_since
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
        }
    }
}
