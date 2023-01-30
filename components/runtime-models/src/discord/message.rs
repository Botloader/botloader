use std::str::FromStr;

use crate::{discord::channel::ChannelType, util::NotBigU64};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::id::Id;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/MessageActivity.ts")]
pub struct MessageActivity {
    pub kind: MessageActivityType,
    pub party_id: Option<String>,
}

impl From<twilight_model::channel::message::MessageActivity> for MessageActivity {
    fn from(v: twilight_model::channel::message::MessageActivity) -> Self {
        Self {
            kind: v.kind.into(),
            party_id: v.party_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/MessageActivityType.ts")]
pub enum MessageActivityType {
    Join,
    Spectate,
    Listen,
    JoinRequest,
}

impl From<twilight_model::channel::message::MessageActivityType> for MessageActivityType {
    fn from(v: twilight_model::channel::message::MessageActivityType) -> Self {
        match v {
            twilight_model::channel::message::MessageActivityType::Join => Self::Join,
            twilight_model::channel::message::MessageActivityType::Spectate => Self::Spectate,
            twilight_model::channel::message::MessageActivityType::Listen => Self::Listen,
            twilight_model::channel::message::MessageActivityType::JoinRequest => Self::JoinRequest,
            _ => todo!(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/MessageApplication.ts")]
pub struct MessageApplication {
    pub cover_image: Option<String>,
    pub description: String,
    pub icon: Option<String>,
    pub id: String,
    pub name: String,
}

impl From<twilight_model::channel::message::MessageApplication> for MessageApplication {
    fn from(v: twilight_model::channel::message::MessageApplication) -> Self {
        Self {
            cover_image: v.cover_image.as_ref().map(ToString::to_string),
            description: v.description,
            icon: v.icon.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            name: v.name,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/Attachment.ts")]
pub struct Attachment {
    pub content_type: Option<String>,
    pub filename: String,
    pub height: Option<i32>,
    pub id: String,
    pub proxy_url: String,
    pub size: NotBigU64,
    pub url: String,
    pub width: Option<i32>,
}

impl From<twilight_model::channel::Attachment> for Attachment {
    fn from(v: twilight_model::channel::Attachment) -> Self {
        Self {
            content_type: v.content_type,
            filename: v.filename,
            height: v.height.map(|v| v as i32),
            id: v.id.to_string(),
            proxy_url: v.proxy_url,
            size: NotBigU64(v.size),
            url: v.url,
            width: v.width.map(|v| v as i32),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/MessageType.ts")]
pub enum MessageType {
    Regular,
    RecipientAdd,
    RecipientRemove,
    Call,
    ChannelNameChange,
    ChannelIconChange,
    ChannelMessagePinned,
    GuildMemberJoin,
    UserPremiumSub,
    UserPremiumSubTier1,
    UserPremiumSubTier2,
    UserPremiumSubTier3,
    ChannelFollowAdd,
    GuildDiscoveryDisqualified,
    GuildDiscoveryRequalified,
    GuildDiscoveryGracePeriodInitialWarning,
    GuildDiscoveryGracePeriodFinalWarning,
    Reply,
    GuildInviteReminder,
    ChatInputCommand,
    ThreadCreated,
    ThreadStarterMessage,
    ContextMenuCommand,
    AutoModerationAction,
    RoleSubscriptionPurchase,
    InteractionPremiumUpsell,
    GuildApplicationPremiumSubscription,
}

impl From<twilight_model::channel::message::MessageType> for MessageType {
    fn from(v: twilight_model::channel::message::MessageType) -> Self {
        use twilight_model::channel::message::MessageType as TwilightMessageType;

        match v {
            TwilightMessageType::Regular => Self::Regular,
            TwilightMessageType::RecipientAdd => Self::RecipientAdd,
            TwilightMessageType::RecipientRemove => Self::RecipientRemove,
            TwilightMessageType::Call => Self::Call,
            TwilightMessageType::ChannelNameChange => Self::ChannelNameChange,
            TwilightMessageType::ChannelIconChange => Self::ChannelIconChange,
            TwilightMessageType::ChannelMessagePinned => Self::ChannelMessagePinned,
            TwilightMessageType::UserJoin => Self::GuildMemberJoin,
            TwilightMessageType::GuildBoost => Self::UserPremiumSub,
            TwilightMessageType::GuildBoostTier1 => Self::UserPremiumSubTier1,
            TwilightMessageType::GuildBoostTier2 => Self::UserPremiumSubTier2,
            TwilightMessageType::GuildBoostTier3 => Self::UserPremiumSubTier3,
            TwilightMessageType::ChannelFollowAdd => Self::ChannelFollowAdd,
            TwilightMessageType::GuildDiscoveryDisqualified => Self::GuildDiscoveryDisqualified,
            TwilightMessageType::GuildDiscoveryRequalified => Self::GuildDiscoveryRequalified,
            TwilightMessageType::GuildDiscoveryGracePeriodInitialWarning => {
                Self::GuildDiscoveryGracePeriodInitialWarning
            }
            TwilightMessageType::GuildDiscoveryGracePeriodFinalWarning => {
                Self::GuildDiscoveryGracePeriodFinalWarning
            }
            TwilightMessageType::Reply => Self::Reply,
            TwilightMessageType::GuildInviteReminder => Self::GuildInviteReminder,
            TwilightMessageType::ThreadCreated => Self::ThreadCreated,
            TwilightMessageType::ThreadStarterMessage => Self::ThreadStarterMessage,
            TwilightMessageType::ContextMenuCommand => Self::ContextMenuCommand,
            TwilightMessageType::ChatInputCommand => Self::ChatInputCommand,
            TwilightMessageType::AutoModerationAction => Self::AutoModerationAction,
            TwilightMessageType::RoleSubscriptionPurchase => Self::RoleSubscriptionPurchase,
            TwilightMessageType::InteractionPremiumUpsell => Self::InteractionPremiumUpsell,
            TwilightMessageType::GuildApplicationPremiumSubscription => {
                Self::GuildApplicationPremiumSubscription
            }
            _ => {
                panic!("unknown message type: {}", u8::from(v));
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/ChannelMention.ts")]
pub struct ChannelMention {
    pub guild_id: String,
    pub id: String,
    pub kind: ChannelType,
    pub name: String,
}

impl From<twilight_model::channel::ChannelMention> for ChannelMention {
    fn from(v: twilight_model::channel::ChannelMention) -> Self {
        Self {
            guild_id: v.guild_id.to_string(),
            id: v.id.to_string(),
            kind: v.kind.into(),
            name: v.name,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/MessageReaction.ts")]
pub struct MessageReaction {
    pub count: NotBigU64,
    pub emoji: ReactionType,
    pub me: bool,
}

impl From<twilight_model::channel::message::Reaction> for MessageReaction {
    fn from(v: twilight_model::channel::message::Reaction) -> Self {
        Self {
            count: NotBigU64(v.count),
            emoji: v.emoji.into(),
            me: v.me,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[ts(export_to = "bindings/discord/ReactionType.ts")]
pub enum ReactionType {
    Custom {
        #[serde(default)]
        animated: bool,
        // Even though it says that the id can be nil in the docs,
        // it is a bit misleading as that should only happen when
        // the reaction is a unicode emoji and then it is caught by
        // the other variant.
        id: String,
        // Name is nil if the emoji data is no longer avaiable, for
        // example if the emoji have been deleted off the guild.
        name: Option<String>,
    },
    Unicode {
        unicode: String,
    },
}

impl From<twilight_model::channel::message::ReactionType> for ReactionType {
    fn from(v: twilight_model::channel::message::ReactionType) -> Self {
        match v {
            twilight_model::channel::message::ReactionType::Custom { animated, name, id } => {
                Self::Custom {
                    animated,
                    name,
                    id: id.to_string(),
                }
            }
            twilight_model::channel::message::ReactionType::Unicode { name } => {
                Self::Unicode { unicode: name }
            }
        }
    }
}

impl From<ReactionType> for twilight_model::channel::message::ReactionType {
    fn from(v: ReactionType) -> Self {
        match v {
            ReactionType::Custom { animated, name, id } => Self::Custom {
                animated,
                name,
                // TODO: maybe we change to TryFrom instead?
                // or just keep it like this and silently fail i guess?
                //
                // Realistically this won't really change the behaviour if the user passes in
                // a invalid custom emoji id
                id: Id::from_str(&id).unwrap_or_else(|_| Id::new(1)),
            },
            ReactionType::Unicode { unicode } => Self::Unicode { name: unicode },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/SendEmoji.ts")]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum SendEmoji {
    Custom {
        // Even though it says that the id can be nil in the docs,
        // it is a bit misleading as that should only happen when
        // the reaction is a unicode emoji and then it is caught by
        // the other variant.
        id: String,
        // Name is nil if the emoji data is no longer avaiable, for
        // example if the emoji have been deleted off the guild.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    Unicode {
        unicode: String,
    },
}

impl<'a> From<&'a SendEmoji>
    for twilight_http::request::channel::reaction::RequestReactionType<'a>
{
    fn from(v: &'a SendEmoji) -> Self {
        match v {
            SendEmoji::Custom { name, id } => Self::Custom {
                name: name.as_deref(),
                // TODO: maybe we change to TryFrom instead?
                // or just keep it like this and silently fail i guess?
                //
                // Realistically this won't really change the behaviour if the user passes in
                // a invalid custom emoji id it will be handled on discord's end and an
                // error will be thrown there, were just changing where were catching the error is all
                id: Id::from_str(id).unwrap_or_else(|_| Id::new(1)),
            },
            SendEmoji::Unicode { unicode } => Self::Unicode { name: unicode },
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/MessageReference.ts")]
pub struct MessageReference {
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub message_id: Option<String>,
    pub fail_if_not_exists: Option<bool>,
}

impl From<twilight_model::channel::message::MessageReference> for MessageReference {
    fn from(v: twilight_model::channel::message::MessageReference) -> Self {
        Self {
            channel_id: v.channel_id.as_ref().map(ToString::to_string),
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            message_id: v.message_id.as_ref().map(ToString::to_string),
            fail_if_not_exists: v.fail_if_not_exists,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/MessageFlags.ts")]
pub struct MessageFlags {
    // #[ts(optional)]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossposted: Option<bool>, //  1 << 0	this message has been published to subscribed channels (via Channel Following)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_crosspost: Option<bool>, //  1 << 1	this message originated from a message in another channel (via Channel Following)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_embeds: Option<bool>, //  1 << 2	do not include any embeds when serializing this message
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_message_deleted: Option<bool>, //  1 << 3	the source message for this crosspost has been deleted (via Channel Following)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urgent: Option<bool>, //  1 << 4	this message came from the urgent message system
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_thread: Option<bool>, //  1 << 5	this message has an associated thread, with the same id as the message
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>, //  1 << 6	this message is only visible to the user who invoked the Interaction
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loading: Option<bool>, //  1 << 7	this message is an Interaction Response and the bot is "thinking"
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_to_mention_some_roles_in_thread: Option<bool>, //  1 << 8	this message failed to mention some roles and add their members to the thread
}

impl From<twilight_model::channel::message::MessageFlags> for MessageFlags {
    fn from(v: twilight_model::channel::message::MessageFlags) -> Self {
        use twilight_model::channel::message::MessageFlags as TwilightMessageFlags;

        Self {
            crossposted: Some(v.contains(TwilightMessageFlags::CROSSPOSTED)),
            is_crosspost: Some(v.contains(TwilightMessageFlags::IS_CROSSPOST)),
            suppress_embeds: Some(v.contains(TwilightMessageFlags::SUPPRESS_EMBEDS)),
            source_message_deleted: Some(v.contains(TwilightMessageFlags::SOURCE_MESSAGE_DELETED)),
            urgent: Some(v.contains(TwilightMessageFlags::URGENT)),
            has_thread: Some(v.contains(TwilightMessageFlags::HAS_THREAD)),
            ephemeral: Some(v.contains(TwilightMessageFlags::EPHEMERAL)),
            loading: Some(v.contains(TwilightMessageFlags::LOADING)),
            failed_to_mention_some_roles_in_thread: Some(
                v.contains(TwilightMessageFlags::FAILED_TO_MENTION_SOME_ROLES_IN_THREAD),
            ),
        }
    }
}

impl From<MessageFlags> for twilight_model::channel::message::MessageFlags {
    fn from(v: MessageFlags) -> Self {
        let mut out = Self::empty();
        if matches!(v.crossposted, Some(true)) {
            out |= Self::CROSSPOSTED;
        }
        if matches!(v.is_crosspost, Some(true)) {
            out |= Self::IS_CROSSPOST;
        }
        if matches!(v.suppress_embeds, Some(true)) {
            out |= Self::SUPPRESS_EMBEDS;
        }
        if matches!(v.source_message_deleted, Some(true)) {
            out |= Self::SOURCE_MESSAGE_DELETED;
        }
        if matches!(v.urgent, Some(true)) {
            out |= Self::URGENT;
        }
        if matches!(v.has_thread, Some(true)) {
            out |= Self::HAS_THREAD;
        }
        if matches!(v.ephemeral, Some(true)) {
            out |= Self::EPHEMERAL;
        }
        if matches!(v.loading, Some(true)) {
            out |= Self::LOADING;
        }
        if matches!(v.failed_to_mention_some_roles_in_thread, Some(true)) {
            out |= Self::FAILED_TO_MENTION_SOME_ROLES_IN_THREAD;
        }

        out
    }
}
