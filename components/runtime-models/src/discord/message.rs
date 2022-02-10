use super::{member::PartialMember, user::User};
use crate::{
    discord::{channel::ChannelType, embed::Embed},
    util::NotBigU64,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/Message.ts")]
pub struct Message {
    pub activity: Option<MessageActivity>,
    pub application: Option<MessageApplication>,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: String,
    pub content: String,
    pub edited_timestamp: Option<NotBigU64>,
    pub embeds: Vec<Embed>,
    pub flags: Option<NotBigU64>,
    pub guild_id: Option<String>,
    pub id: String,
    pub kind: MessageType,
    pub member: Option<PartialMember>,
    pub mention_channels: Vec<ChannelMention>,
    pub mention_everyone: bool,
    pub mention_roles: Vec<String>,
    pub mentions: Vec<Mention>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub reference: Option<MessageReference>,
    pub referenced_message: Option<Box<Message>>,
    pub timestamp: NotBigU64,
    pub tts: bool,
    pub webhook_id: Option<String>,
}

impl From<twilight_model::channel::Message> for Message {
    fn from(v: twilight_model::channel::Message) -> Self {
        Self {
            activity: v.activity.map(From::from),
            application: v.application.map(From::from),
            attachments: v.attachments.into_iter().map(From::from).collect(),
            author: v.author.into(),
            channel_id: v.channel_id.to_string(),
            content: v.content,
            edited_timestamp: v
                .edited_timestamp
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            embeds: v.embeds.into_iter().map(From::from).collect(),
            flags: v.flags.map(|f| NotBigU64(f.bits())),
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            kind: v.kind.into(),
            member: v.member.map(From::from),
            mention_channels: v.mention_channels.into_iter().map(From::from).collect(),
            mention_everyone: v.mention_everyone,
            mention_roles: v.mention_roles.iter().map(ToString::to_string).collect(),
            mentions: v.mentions.into_iter().map(From::from).collect(),
            pinned: v.pinned,
            reactions: v.reactions.into_iter().map(From::from).collect(),
            reference: v.reference.map(From::from),
            referenced_message: v.referenced_message.map(|e| Box::new((*e).into())),
            timestamp: NotBigU64(v.timestamp.as_micros() as u64 / 1000),
            tts: v.tts,
            webhook_id: v.webhook_id.as_ref().map(ToString::to_string),
        }
    }
}

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
            cover_image: v.cover_image,
            description: v.description,
            icon: v.icon,
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
            TwilightMessageType::GuildMemberJoin => Self::GuildMemberJoin,
            TwilightMessageType::UserPremiumSub => Self::UserPremiumSub,
            TwilightMessageType::UserPremiumSubTier1 => Self::UserPremiumSubTier1,
            TwilightMessageType::UserPremiumSubTier2 => Self::UserPremiumSubTier2,
            TwilightMessageType::UserPremiumSubTier3 => Self::UserPremiumSubTier3,
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
#[ts(export_to = "bindings/discord/Mention.ts")]
pub struct Mention {
    /// Hash of the user's avatar, if any.
    pub avatar: Option<String>,
    /// Whether the user is a bot.
    pub bot: bool,
    /// Discriminator used to differentiate people with the same username.
    ///
    /// # serde
    ///
    /// The discriminator field can be deserialized from either a string or an
    /// integer. The field will always serialize into a string due to that being
    /// the type Discord's API uses.
    pub discriminator: u16,
    /// Unique ID of the user.
    pub id: String,
    /// Member object for the user in the guild, if available.
    pub member: Option<PartialMember>,
    /// Username of the user.
    pub username: String,
    /// Public flags on the user's account.
    pub public_flags: NotBigU64,
}

impl From<twilight_model::channel::message::Mention> for Mention {
    fn from(v: twilight_model::channel::message::Mention) -> Self {
        Self {
            avatar: v.avatar,
            bot: v.bot,
            discriminator: v.discriminator,
            id: v.id.to_string(),
            member: v.member.map(From::from),
            username: v.name,
            public_flags: NotBigU64(v.public_flags.bits()),
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

impl From<twilight_model::channel::message::MessageReaction> for MessageReaction {
    fn from(v: twilight_model::channel::message::MessageReaction) -> Self {
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
#[serde(tag = "kind")]
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
        name: String,
    },
}

impl From<twilight_model::channel::ReactionType> for ReactionType {
    fn from(v: twilight_model::channel::ReactionType) -> Self {
        match v {
            twilight_model::channel::ReactionType::Custom { animated, name, id } => Self::Custom {
                animated,
                name,
                id: id.to_string(),
            },
            twilight_model::channel::ReactionType::Unicode { name } => Self::Unicode { name },
        }
    }
}

// impl From<ReactionType> for twilight_model::channel::ReactionType {
//     fn from(v: ReactionType) -> Self {
//         match v {
//             ReactionType::Custom { animated, name, id } => Self::Custom { animated, name, id },
//             ReactionType::Unicode { name } => Self::Unicode { name },
//         }
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
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

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/StickerType.ts")]
pub enum StickerType {
    /// Official sticker in a pack.
    ///
    /// Part of nitro or in a removed purchasable pack.
    Standard = 1,
    /// Sticker uploaded to a boosted guild for the guild's members.
    Guild = 2,
}

impl From<twilight_model::channel::message::sticker::StickerType> for StickerType {
    fn from(v: twilight_model::channel::message::sticker::StickerType) -> Self {
        match v {
            twilight_model::channel::message::sticker::StickerType::Standard => Self::Standard,
            twilight_model::channel::message::sticker::StickerType::Guild => Self::Guild,
        }
    }
}

impl From<StickerType> for twilight_model::channel::message::sticker::StickerType {
    fn from(v: StickerType) -> Self {
        match v {
            StickerType::Standard => Self::Standard,
            StickerType::Guild => Self::Guild,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/Sticker.ts")]
pub struct Sticker {
    /// Whether the sticker is available.
    pub available: bool,
    /// Description of the sticker.
    pub description: Option<String>,
    /// Format type.
    pub format_type: StickerFormatType,
    /// ID of the guild that owns the sticker.
    pub guild_id: Option<String>,
    /// Unique ID of the sticker.
    pub id: String,
    /// Name of the sticker.
    pub name: String,
    /// Unique ID of the pack the sticker is in.
    pub pack_id: Option<String>,
    /// Sticker's sort order within a pack.
    pub sort_value: Option<NotBigU64>,
    /// CSV list of tags the sticker is assigned to, if any.
    pub tags: String,
    /// ID of the user that uploaded the sticker.
    pub user: Option<User>,

    pub kind: StickerType,
}

impl From<twilight_model::channel::message::Sticker> for Sticker {
    fn from(v: twilight_model::channel::message::Sticker) -> Self {
        Self {
            description: v.description,
            format_type: v.format_type.into(),
            id: v.id.to_string(),
            name: v.name,
            pack_id: v.pack_id.as_ref().map(ToString::to_string),
            tags: v.tags,
            available: v.available,
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            sort_value: v.sort_value.map(NotBigU64),
            user: v.user.map(|u| u.into()),
            kind: v.kind.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/StickerFormatType.ts")]
pub enum StickerFormatType {
    /// Sticker format is a PNG.
    Png,
    /// Sticker format is an APNG.
    Apng,
    /// Sticker format is a LOTTIE.
    Lottie,
}

impl From<twilight_model::channel::message::sticker::StickerFormatType> for StickerFormatType {
    fn from(v: twilight_model::channel::message::sticker::StickerFormatType) -> Self {
        match v {
            twilight_model::channel::message::sticker::StickerFormatType::Apng => Self::Apng,
            twilight_model::channel::message::sticker::StickerFormatType::Png => Self::Png,
            twilight_model::channel::message::sticker::StickerFormatType::Lottie => Self::Lottie,
        }
    }
}
impl From<StickerFormatType> for twilight_model::channel::message::sticker::StickerFormatType {
    fn from(v: StickerFormatType) -> Self {
        match v {
            StickerFormatType::Apng => Self::Apng,
            StickerFormatType::Png => Self::Png,
            StickerFormatType::Lottie => Self::Lottie,
        }
    }
}
