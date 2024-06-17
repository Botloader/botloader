use crate::{
    discord::{
        component::Component,
        embed::Embed,
        member::PartialMember,
        message::{
            Attachment, ChannelMention, MessageActivity, MessageApplication, MessageFlags,
            MessageReaction, MessageReference, MessageType,
        },
    },
    internal::user::User,
    util::NotBigU64,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::{
    channel::message::{
        AllowedMentions as TwilightAllowedMentions, MentionType as TwilightParseTypes,
    },
    id::Id,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/DeleteMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpDeleteMessage {
    pub channel_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/DeleteMessagesBulk.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpDeleteMessagesBulk {
    pub channel_id: String,
    pub message_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateChannelMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateChannelMessage {
    pub channel_id: String,
    pub fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/EditChannelMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpEditChannelMessage {
    pub channel_id: String,
    pub message_id: String,
    pub fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateFollowUpMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateFollowUpMessage {
    pub interaction_token: String,
    pub fields: OpCreateMessageFields,
    pub flags: Option<MessageFlags>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CreateMessageFields.ts")]
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
    #[serde(default)]
    #[ts(optional)]
    pub components: Option<Vec<Component>>,
    #[serde(default)]
    #[ts(optional)]
    pub reply_to_message_id: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub attachments: Option<Vec<OpCreateMessageAttachment>>,
}

// Converts attachments from our internal runtime_models to twilight's models
// Needed because we store data in b64 in our translation layer
pub fn convert_attachments(
    internal: Vec<OpCreateMessageAttachment>,
) -> Result<Vec<twilight_model::http::attachment::Attachment>, anyhow::Error> {
    Ok(internal
        .into_iter()
        .map(|v| {
            let decoded = match v.decode_data() {
                Ok(d) => d,
                Err(err) => return Err(err),
            };

            Ok(twilight_model::http::attachment::Attachment {
                description: v.description,
                file: decoded,
                filename: v.filename,
                id: v.id.0,
            })
        })
        .collect::<Result<Vec<_>, _>>()?)
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/OpCreateMessageAttachment.ts")]
#[serde(rename_all = "camelCase")]
pub struct OpCreateMessageAttachment {
    pub description: Option<String>,
    pub data_b64: String,
    pub filename: String,
    pub id: NotBigU64,
}

impl OpCreateMessageAttachment {
    pub fn decode_data(&self) -> Result<Vec<u8>, base64_simd::Error> {
        base64_simd::forgiving_decode_to_vec(self.data_b64.as_bytes())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/AllowedMentions.ts")]
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
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
            roles: v
                .roles
                .iter()
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
            replied_user: v.replied_user,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/MentionParseTypes.ts")]
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
#[ts(export_to = "bindings/internal/GetMessages.ts")]
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

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IMessage")]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/IMessage.ts")]
pub struct Message {
    pub activity: Option<MessageActivity>,
    pub application: Option<MessageApplication>,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: String,
    pub content: String,
    pub components: Vec<Component>,
    pub edited_timestamp: Option<NotBigU64>,
    pub embeds: Vec<Embed>,
    pub flags: Option<MessageFlags>,
    pub guild_id: Option<String>,
    pub id: String,
    pub kind: MessageType,
    pub member: Option<PartialMember>,
    pub mention_channels: Vec<ChannelMention>,
    pub mention_everyone: bool,
    pub mention_roles: Vec<String>,
    pub mentions: Vec<UserMention>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub reference: Option<MessageReference>,
    pub referenced_message: Option<Box<Message>>,
    pub timestamp: NotBigU64,
    pub tts: bool,
    pub webhook_id: Option<String>,
}

impl TryFrom<twilight_model::channel::Message> for Message {
    type Error = anyhow::Error;

    fn try_from(v: twilight_model::channel::Message) -> Result<Self, Self::Error> {
        Ok(Self {
            activity: v.activity.map(From::from),
            application: v.application.map(From::from),
            attachments: v.attachments.into_iter().map(From::from).collect(),
            author: v.author.into(),
            channel_id: v.channel_id.to_string(),
            content: v.content,
            components: v.components.into_iter().map(Into::into).collect(),
            edited_timestamp: v
                .edited_timestamp
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            embeds: v.embeds.into_iter().map(From::from).collect(),
            flags: v.flags.map(From::from),
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            kind: v.kind.try_into()?,
            member: v.member.map(From::from),
            mention_channels: v.mention_channels.into_iter().map(From::from).collect(),
            mention_everyone: v.mention_everyone,
            mention_roles: v.mention_roles.iter().map(ToString::to_string).collect(),
            mentions: v.mentions.into_iter().map(From::from).collect(),
            pinned: v.pinned,
            reactions: v.reactions.into_iter().map(From::from).collect(),
            reference: v.reference.map(From::from),
            referenced_message: v
                .referenced_message
                .map(|e| (*e).try_into())
                .transpose()?
                .map(Box::new),
            timestamp: NotBigU64(v.timestamp.as_micros() as u64 / 1000),
            tts: v.tts,
            webhook_id: v.webhook_id.as_ref().map(ToString::to_string),
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IUserMention")]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/UserMention.ts")]
pub struct UserMention {
    user: User,
    member: Option<PartialMember>,
}

impl From<twilight_model::channel::message::Mention> for UserMention {
    fn from(v: twilight_model::channel::message::Mention) -> Self {
        Self {
            user: User {
                avatar: v.avatar.as_ref().map(ToString::to_string),
                bot: v.bot,
                discriminator: v.discriminator().to_string(),
                id: v.id.to_string(),
                public_flags: Some(v.public_flags.into()),
                username: v.name,
                locale: None,
                premium_type: None,
                system: None,
                global_name: None,
                banner: None,
                accent_color: None,
            },
            member: v.member.map(From::from),
        }
    }
}
