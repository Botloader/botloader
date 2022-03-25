use serde::Serialize;
use ts_rs::TS;

use crate::{
    discord::channel::{ChannelType, PermissionOverwrite, ThreadMetadata, VideoQualityMode},
    internal::member::Member,
    util::NotBigU64,
};

#[derive(Clone, Debug, Serialize, TS)]
#[serde(untagged)]
#[ts(export, rename = "InternalGuildChannel")]
#[ts(export_to = "bindings/internal/GuildChannel.ts")]
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
#[ts(export, rename = "IVoiceChannel")]
#[ts(export_to = "bindings/internal/VoiceChannel.ts")]
#[serde(rename_all = "camelCase")]
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
#[ts(export, rename = "ITextChannel")]
#[ts(export_to = "bindings/internal/TextChannel.ts")]
#[serde(rename_all = "camelCase")]
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
#[ts(export, rename = "IPublicThread")]
#[ts(export_to = "bindings/internal/PublicThread.ts")]
#[serde(rename_all = "camelCase")]
pub struct PublicThread {
    pub default_auto_archive_duration_minutes: Option<u32>,
    pub id: String,
    #[ts(type = "'PublicThread'")]
    pub kind: ChannelType,
    pub member: Option<SelfThreadMember>,
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
#[ts(export, rename = "IPrivateThread")]
#[ts(export_to = "bindings/internal/PrivateThread.ts")]
#[serde(rename_all = "camelCase")]
pub struct PrivateThread {
    pub default_auto_archive_duration_minutes: Option<u32>,
    pub id: String,
    pub invitable: Option<bool>,
    #[ts(type = "'PrivateThread'")]
    pub kind: ChannelType,
    pub member: Option<SelfThreadMember>,
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
#[ts(export, rename = "INewsThread")]
#[ts(export_to = "bindings/internal/NewsThread.ts")]
#[serde(rename_all = "camelCase")]
pub struct NewsThread {
    pub default_auto_archive_duration_minutes: Option<u32>,
    pub id: String,
    #[ts(type = "'NewsThread'")]
    pub kind: ChannelType,
    pub member: Option<SelfThreadMember>,
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
#[ts(export, rename = "IThreadMember")]
#[ts(export_to = "bindings/internal/ThreadMember.ts")]
#[serde(rename_all = "camelCase")]
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
#[ts(export, rename = "ISelfThreadMember")]
#[ts(export_to = "bindings/internal/ISelfThreadMember.ts")]
#[serde(rename_all = "camelCase")]
pub struct SelfThreadMember {
    // Removed as the values aren't documented anywhere and i want to make a proper
    // abstraction for this similar to UserFlags and the like.
    // pub flags: NotBigU64,
    pub join_timestamp: NotBigU64,
}

impl From<twilight_model::channel::thread::ThreadMember> for SelfThreadMember {
    fn from(v: twilight_model::channel::thread::ThreadMember) -> Self {
        Self {
            // flags: NotBigU64(v.flags),
            join_timestamp: NotBigU64(v.join_timestamp.as_micros() as u64 / 1000),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "ICategoryChannel")]
#[ts(export_to = "bindings/internal/CategoryChannel.ts")]
#[serde(rename_all = "camelCase")]
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
