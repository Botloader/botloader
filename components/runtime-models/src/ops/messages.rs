use crate::discord::embed::Embed;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::{
    channel::message::{
        allowed_mentions::ParseTypes as TwilightParseTypes,
        AllowedMentions as TwilightAllowedMentions,
    },
    id::{RoleId, UserId},
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/DeleteMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpDeleteMessage {
    pub channel_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/DeleteMessagesBulk.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpDeleteMessagesBulk {
    pub channel_id: String,
    pub message_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/CreateChannelMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateChannelMessage {
    pub channel_id: String,
    pub fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/EditChannelMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpEditChannelMessage {
    pub channel_id: String,
    pub message_id: String,
    pub fields: OpEditMessageFields,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/CreateFollowUpMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateFollowUpMessage {
    pub interaction_token: String,
    pub fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/CreateMessageFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateMessageFields {
    #[serde(default)]
    #[ts(optional)]
    pub content: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub embeds: Option<Vec<Embed>>,
    #[serde(default)]
    #[ts(optional)]
    pub allowed_mentions: Option<AllowedMentions>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/EditMessageFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpEditMessageFields {
    #[serde(default)]
    #[ts(optional)]
    pub content: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub embeds: Option<Vec<Embed>>,
    #[serde(default)]
    #[ts(optional)]
    pub allowed_mentions: Option<AllowedMentions>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/AllowedMentions.ts")]
#[serde(rename_all = "camelCase")]
pub struct AllowedMentions {
    parse: Vec<MentionParseTypes>,
    users: Vec<String>,
    roles: Vec<String>,
    replied_user: bool,
}

impl From<AllowedMentions> for TwilightAllowedMentions {
    fn from(v: AllowedMentions) -> Self {
        Self {
            parse: v.parse.into_iter().map(Into::into).collect(),
            users: v
                .users
                .iter()
                .filter_map(|s| UserId::new_checked(s.parse().ok()?))
                .collect(),
            roles: v
                .roles
                .iter()
                .filter_map(|s| RoleId::new_checked(s.parse().ok()?))
                .collect(),
            replied_user: v.replied_user,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/MentionParseTypes.ts")]
pub enum MentionParseTypes {
    Everyone,
    Roles,
    Users,
}

impl From<MentionParseTypes> for TwilightParseTypes {
    fn from(pt: MentionParseTypes) -> Self {
        match pt {
            MentionParseTypes::Everyone => Self::Everyone,
            MentionParseTypes::Roles => Self::Roles,
            MentionParseTypes::Users => Self::Users,
        }
    }
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/GetMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpGetMessage {
    pub channel_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/ops/GetMessages.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpGetMessages {
    pub channel_id: String,

    #[serde(default)]
    #[ts(optional)]
    pub after: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub before: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<i32>,
}
