use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{discord::member::Member, util::NotBigU64};

#[derive(Clone, Debug, Serialize, TS)]
#[serde(untagged)]
#[ts(export)]
#[ts(export_to = "bindings/discord/GuildChannel.ts")]
pub enum GuildChannel {
    Category(CategoryChannel),
    NewsThread(Box<NewsThread>),
    PrivateThread(Box<PrivateThread>),
    PublicThread(Box<PublicThread>),
    Text(TextChannel),
    Voice(VoiceChannel),
    Stage(VoiceChannel),
}

impl From<twilight_model::channel::GuildChannel> for GuildChannel {
    fn from(v: twilight_model::channel::GuildChannel) -> Self {
        match v {
            twilight_model::channel::GuildChannel::Category(c) => Self::Category(c.into()),
            twilight_model::channel::GuildChannel::NewsThread(ns) => {
                Self::NewsThread(Box::new(ns.into()))
            }
            twilight_model::channel::GuildChannel::PrivateThread(c) => {
                Self::PrivateThread(Box::new(c.into()))
            }
            twilight_model::channel::GuildChannel::PublicThread(c) => {
                Self::PublicThread(Box::new(c.into()))
            }
            twilight_model::channel::GuildChannel::Text(c) => Self::Text(c.into()),
            twilight_model::channel::GuildChannel::Voice(c) => Self::Voice(c.into()),
            twilight_model::channel::GuildChannel::Stage(c) => Self::Stage(c.into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/VoiceChannel.ts")]
pub struct VoiceChannel {
    pub bitrate: NotBigU64,
    pub id: String,
    #[ts(type = "'Voice'|'StageVoice'")]
    pub kind: ChannelType,
    pub name: String,
    pub parent_id: Option<String>,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub position: i64,
    pub rtc_region: Option<String>,
    pub user_limit: Option<NotBigU64>,
    pub video_quality_mode: Option<VideoQualityMode>,
}

impl From<twilight_model::channel::VoiceChannel> for VoiceChannel {
    fn from(v: twilight_model::channel::VoiceChannel) -> Self {
        Self {
            bitrate: NotBigU64(v.bitrate),
            id: v.id.to_string(),
            kind: v.kind.into(),
            name: v.name,
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            permission_overwrites: v
                .permission_overwrites
                .into_iter()
                .map(Into::into)
                .collect(),
            position: v.position,
            rtc_region: v.rtc_region,
            user_limit: v.user_limit.map(NotBigU64),
            video_quality_mode: v.video_quality_mode.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/VideoQualityMode.ts")]
pub enum VideoQualityMode {
    Auto,
    Full,
}

impl From<twilight_model::channel::VideoQualityMode> for VideoQualityMode {
    fn from(v: twilight_model::channel::VideoQualityMode) -> Self {
        match v {
            twilight_model::channel::VideoQualityMode::Auto => Self::Auto,
            twilight_model::channel::VideoQualityMode::Full => Self::Full,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/TextChannel.ts")]
pub struct TextChannel {
    pub id: String,
    #[ts(type = "'Text'|'News'|'Store'")]
    pub kind: ChannelType,
    pub last_pin_timestamp: Option<NotBigU64>,
    pub name: String,
    pub nsfw: bool,
    pub parent_id: Option<String>,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub position: i64,
    pub rate_limit_per_user: Option<NotBigU64>,
    pub topic: Option<String>,
}

impl From<twilight_model::channel::TextChannel> for TextChannel {
    fn from(v: twilight_model::channel::TextChannel) -> Self {
        Self {
            id: v.id.to_string(),
            kind: v.kind.into(),
            last_pin_timestamp: v
                .last_pin_timestamp
                .map(|e| NotBigU64(e.as_micros() as u64 / 1000)),
            name: v.name,
            nsfw: v.nsfw,
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            permission_overwrites: v
                .permission_overwrites
                .into_iter()
                .map(Into::into)
                .collect(),
            position: v.position,
            rate_limit_per_user: v.rate_limit_per_user.map(NotBigU64),
            topic: v.topic,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/PublicThread.ts")]
pub struct PublicThread {
    pub default_auto_archive_duration_minutes: Option<u32>,
    pub id: String,
    #[ts(type = "'PublicThread'")]
    pub kind: ChannelType,
    pub member: Option<ThreadMember>,
    pub member_count: u8,
    pub message_count: u8,
    pub name: String,
    pub owner_id: Option<String>,
    pub parent_id: Option<String>,
    pub rate_limit_per_user: Option<NotBigU64>,
    pub thread_metadata: ThreadMetadata,
}

impl From<twilight_model::channel::thread::PublicThread> for PublicThread {
    fn from(v: twilight_model::channel::thread::PublicThread) -> Self {
        Self {
            default_auto_archive_duration_minutes: v
                .default_auto_archive_duration
                .map(|v| v.number() as u32),
            id: v.id.to_string(),
            kind: v.kind.into(),
            member: v.member.map(Into::into),
            member_count: v.member_count,
            message_count: v.message_count,
            name: v.name,
            owner_id: v.owner_id.as_ref().map(ToString::to_string),
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            rate_limit_per_user: v.rate_limit_per_user.map(NotBigU64),
            thread_metadata: v.thread_metadata.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/PrivateThread.ts")]
pub struct PrivateThread {
    pub default_auto_archive_duration_minutes: Option<u32>,
    pub id: String,
    pub invitable: Option<bool>,
    #[ts(type = "'PrivateThread'")]
    pub kind: ChannelType,
    pub member: Option<ThreadMember>,
    pub member_count: u8,
    pub message_count: u8,
    pub name: String,
    pub owner_id: Option<String>,
    pub parent_id: Option<String>,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub rate_limit_per_user: Option<NotBigU64>,
    pub thread_metadata: ThreadMetadata,
}

impl From<twilight_model::channel::thread::PrivateThread> for PrivateThread {
    fn from(v: twilight_model::channel::thread::PrivateThread) -> Self {
        Self {
            default_auto_archive_duration_minutes: v
                .default_auto_archive_duration
                .map(|v| v.number() as u32),
            id: v.id.to_string(),
            kind: v.kind.into(),
            member: v.member.map(Into::into),
            member_count: v.member_count,
            message_count: v.message_count,
            name: v.name,
            owner_id: v.owner_id.as_ref().map(ToString::to_string),
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            rate_limit_per_user: v.rate_limit_per_user.map(NotBigU64),
            thread_metadata: v.thread_metadata.into(),
            permission_overwrites: v
                .permission_overwrites
                .into_iter()
                .map(Into::into)
                .collect(),
            invitable: v.invitable,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/NewsThread.ts")]
pub struct NewsThread {
    pub default_auto_archive_duration_minutes: Option<u32>,
    pub id: String,
    #[ts(type = "'NewsThread'")]
    pub kind: ChannelType,
    pub member: Option<ThreadMember>,
    pub member_count: u8,
    pub message_count: u8,
    pub name: String,
    pub owner_id: Option<String>,
    pub parent_id: Option<String>,
    pub rate_limit_per_user: Option<NotBigU64>,
    pub thread_metadata: ThreadMetadata,
}

impl From<twilight_model::channel::thread::NewsThread> for NewsThread {
    fn from(v: twilight_model::channel::thread::NewsThread) -> Self {
        Self {
            default_auto_archive_duration_minutes: v
                .default_auto_archive_duration
                .map(|v| v.number() as u32),
            id: v.id.to_string(),
            kind: v.kind.into(),
            member: v.member.map(Into::into),
            member_count: v.member_count,
            message_count: v.message_count,
            name: v.name,
            owner_id: v.owner_id.as_ref().map(ToString::to_string),
            parent_id: v.parent_id.as_ref().map(ToString::to_string),
            rate_limit_per_user: v.rate_limit_per_user.map(NotBigU64),
            thread_metadata: v.thread_metadata.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ThreadMember.ts")]
pub struct ThreadMember {
    // Removed as the values aren't documented anywhere and i want to make a proper
    // abstraction for this similar to UserFlags and the like.
    // pub flags: NotBigU64,
    pub id: Option<String>,
    pub join_timestamp: NotBigU64,
    pub member: Option<Member>,

    // Unsure if presence is provided without presence intent
    // pub presence: Option<Presence>,
    pub user_id: Option<String>,
}

impl From<twilight_model::channel::thread::ThreadMember> for ThreadMember {
    fn from(v: twilight_model::channel::thread::ThreadMember) -> Self {
        Self {
            // flags: NotBigU64(v.flags),
            id: v.id.as_ref().map(ToString::to_string),
            join_timestamp: NotBigU64(v.join_timestamp.as_micros() as u64 / 1000),
            member: v.member.map(Into::into),
            user_id: v.user_id.as_ref().map(ToString::to_string),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ThreadMetadata.ts")]
pub struct ThreadMetadata {
    pub archived: bool,
    pub auto_archive_duration_minutes: u32,
    pub archive_timestamp: NotBigU64,
    pub invitable: Option<bool>,
    pub locked: bool,
}

impl From<twilight_model::channel::thread::ThreadMetadata> for ThreadMetadata {
    fn from(v: twilight_model::channel::thread::ThreadMetadata) -> Self {
        Self {
            archived: v.archived,
            auto_archive_duration_minutes: v.auto_archive_duration.number() as u32,
            archive_timestamp: NotBigU64(v.archive_timestamp.as_micros() as u64 / 1000),
            invitable: v.invitable,
            locked: v.locked,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/CategoryChannel.ts")]
pub struct CategoryChannel {
    pub id: String,
    #[ts(type = "'Category'")]
    pub kind: ChannelType,
    pub name: String,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub position: i64,
}

impl From<twilight_model::channel::CategoryChannel> for CategoryChannel {
    fn from(v: twilight_model::channel::CategoryChannel) -> Self {
        Self {
            kind: v.kind.into(),
            id: v.id.to_string(),
            name: v.name,
            position: v.position,
            permission_overwrites: v
                .permission_overwrites
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ChannelType.ts")]
pub enum ChannelType {
    Text,
    Voice,
    Category,
    News,
    Store,
    StageVoice,
    NewsThread,
    PublicThread,
    PrivateThread,
}

impl From<twilight_model::channel::ChannelType> for ChannelType {
    fn from(v: twilight_model::channel::ChannelType) -> Self {
        match v {
            twilight_model::channel::ChannelType::GuildText => Self::Text,
            twilight_model::channel::ChannelType::GuildVoice => Self::Voice,
            twilight_model::channel::ChannelType::GuildCategory => Self::Category,
            twilight_model::channel::ChannelType::GuildNews => Self::News,
            twilight_model::channel::ChannelType::GuildStore => Self::Store,
            twilight_model::channel::ChannelType::GuildStageVoice => Self::StageVoice,
            twilight_model::channel::ChannelType::GuildNewsThread => Self::NewsThread,
            twilight_model::channel::ChannelType::GuildPublicThread => Self::PublicThread,
            twilight_model::channel::ChannelType::GuildPrivateThread => Self::PrivateThread,
            twilight_model::channel::ChannelType::Group => panic!("unspported channel type: group"),
            twilight_model::channel::ChannelType::Private => {
                panic!("unspported channel type: private")
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/PermissionOverwrite.ts")]
pub struct PermissionOverwrite {
    pub allow_raw: String,
    pub deny_raw: String,
    pub kind: PermissionOverwriteType,
    pub id: String,
}

impl From<twilight_model::channel::permission_overwrite::PermissionOverwrite>
    for PermissionOverwrite
{
    fn from(v: twilight_model::channel::permission_overwrite::PermissionOverwrite) -> Self {
        Self {
            id: match v.kind {
                twilight_model::channel::permission_overwrite::PermissionOverwriteType::Member(
                    id,
                ) => id.to_string(),
                twilight_model::channel::permission_overwrite::PermissionOverwriteType::Role(
                    id,
                ) => id.to_string(),
            },
            allow_raw: v.allow.bits().to_string(),
            deny_raw: v.deny.bits().to_string(),
            kind: v.kind.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/PermissionOverwriteType.ts")]
pub enum PermissionOverwriteType {
    Member,
    Role,
}

impl From<twilight_model::channel::permission_overwrite::PermissionOverwriteType>
    for PermissionOverwriteType
{
    fn from(v: twilight_model::channel::permission_overwrite::PermissionOverwriteType) -> Self {
        match v {
            twilight_model::channel::permission_overwrite::PermissionOverwriteType::Member(_) => {
                Self::Member
            }
            twilight_model::channel::permission_overwrite::PermissionOverwriteType::Role(_) => {
                Self::Role
            }
        }
    }
}
