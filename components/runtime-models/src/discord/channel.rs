use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{discord::member::Member, util::NotBigU64};

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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

// manually implemented for now cause of a bug in the lib
impl ts_rs::TS for GuildChannel {
    const EXPORT_TO: Option<&'static str> = Some("bindings/discord/GuildChannel.ts");
    fn decl() -> String {
        format!("type {}{} = {};", "GuildChannel", "", Self::inline())
    }
    fn name() -> String {
        "GuildChannel".to_owned()
    }
    fn inline() -> String {
        vec![
            <CategoryChannel as ts_rs::TS>::name(),
            <Box<NewsThread> as ts_rs::TS>::name_with_type_args(vec![
                <NewsThread as ts_rs::TS>::name(),
            ]),
            <Box<PrivateThread> as ts_rs::TS>::name_with_type_args(vec![
                <PrivateThread as ts_rs::TS>::name(),
            ]),
            <Box<PublicThread> as ts_rs::TS>::name_with_type_args(vec![
                <PublicThread as ts_rs::TS>::name(),
            ]),
            <TextChannel as ts_rs::TS>::name(),
            <VoiceChannel as ts_rs::TS>::name(),
            <VoiceChannel as ts_rs::TS>::name(),
        ]
        .join(" | ")
    }

    fn dependencies() -> Vec<ts_rs::Dependency> {
        {
            let mut dependencies = vec![];
            if <CategoryChannel as ts_rs::TS>::transparent() {
                dependencies.append(&mut <CategoryChannel as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<CategoryChannel>() {
                dependencies.push(dep);
            }
            if <Box<NewsThread> as ts_rs::TS>::transparent() {
                dependencies.append(&mut <Box<NewsThread> as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<Box<NewsThread>>() {
                dependencies.push(dep);
            }
            if <NewsThread as ts_rs::TS>::transparent() {
                dependencies.append(&mut <NewsThread as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<NewsThread>() {
                dependencies.push(dep);
            }
            if <Box<PrivateThread> as ts_rs::TS>::transparent() {
                dependencies.append(&mut <Box<PrivateThread> as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<Box<PrivateThread>>() {
                dependencies.push(dep);
            }
            if <PrivateThread as ts_rs::TS>::transparent() {
                dependencies.append(&mut <PrivateThread as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<PrivateThread>() {
                dependencies.push(dep);
            }
            if <Box<PublicThread> as ts_rs::TS>::transparent() {
                dependencies.append(&mut <Box<PublicThread> as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<Box<PublicThread>>() {
                dependencies.push(dep);
            }
            if <PublicThread as ts_rs::TS>::transparent() {
                dependencies.append(&mut <PublicThread as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<PublicThread>() {
                dependencies.push(dep);
            }
            if <TextChannel as ts_rs::TS>::transparent() {
                dependencies.append(&mut <TextChannel as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<TextChannel>() {
                dependencies.push(dep);
            }
            if <VoiceChannel as ts_rs::TS>::transparent() {
                dependencies.append(&mut <VoiceChannel as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<VoiceChannel>() {
                dependencies.push(dep);
            }
            if <VoiceChannel as ts_rs::TS>::transparent() {
                dependencies.append(&mut <VoiceChannel as ts_rs::TS>::dependencies());
            } else if let Some(dep) = ts_rs::Dependency::from_ty::<VoiceChannel>() {
                dependencies.push(dep);
            }
            dependencies
        }
    }
    fn transparent() -> bool {
        false
    }
}
#[cfg(test)]
#[test]
fn export_bindings_guildchannel() {
    <GuildChannel as ts_rs::TS>::export().expect("could not export type");
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/VoiceChannel.ts")]
pub struct VoiceChannel {
    pub bitrate: NotBigU64,
    pub guild_id: String,
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
            guild_id: v
                .guild_id
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
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
    pub guild_id: String,
    pub id: String,
    #[ts(type = "'Text'|'News'|'Store'")]
    pub kind: ChannelType,
    pub last_message_id: Option<String>,
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
            guild_id: v
                .guild_id
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            id: v.id.to_string(),
            kind: v.kind.into(),
            last_message_id: v.last_message_id.as_ref().map(ToString::to_string),
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
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    pub guild_id: String,
    pub id: String,
    #[ts(type = "'PublicThread'")]
    pub kind: ChannelType,
    pub last_message_id: Option<String>,
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
            default_auto_archive_duration: v.default_auto_archive_duration.map(Into::into),
            guild_id: v
                .guild_id
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            id: v.id.to_string(),
            kind: v.kind.into(),
            last_message_id: v.last_message_id.as_ref().map(ToString::to_string),
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
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    pub guild_id: String,
    pub id: String,
    pub invitable: Option<bool>,
    #[ts(type = "'PrivateThread'")]
    pub kind: ChannelType,
    pub last_message_id: Option<String>,
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
            default_auto_archive_duration: v.default_auto_archive_duration.map(Into::into),
            guild_id: v
                .guild_id
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            id: v.id.to_string(),
            kind: v.kind.into(),
            last_message_id: v.last_message_id.as_ref().map(ToString::to_string),
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
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    pub guild_id: String,
    pub id: String,
    #[ts(type = "'NewsThread'")]
    pub kind: ChannelType,
    pub last_message_id: Option<String>,
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
            default_auto_archive_duration: v.default_auto_archive_duration.map(Into::into),
            guild_id: v
                .guild_id
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            id: v.id.to_string(),
            kind: v.kind.into(),
            last_message_id: v.last_message_id.as_ref().map(ToString::to_string),
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
#[ts(export_to = "bindings/discord/AutoArchiveDuration.ts")]
pub enum AutoArchiveDuration {
    Hour,
    Day,
    ThreeDays,
    Week,
    Unknown { value: u16 },
}

impl From<twilight_model::channel::thread::AutoArchiveDuration> for AutoArchiveDuration {
    fn from(v: twilight_model::channel::thread::AutoArchiveDuration) -> Self {
        match v {
            twilight_model::channel::thread::AutoArchiveDuration::Hour => Self::Hour,
            twilight_model::channel::thread::AutoArchiveDuration::Day => Self::Day,
            twilight_model::channel::thread::AutoArchiveDuration::ThreeDays => Self::ThreeDays,
            twilight_model::channel::thread::AutoArchiveDuration::Week => Self::Week,
            twilight_model::channel::thread::AutoArchiveDuration::Unknown { value } => {
                Self::Unknown { value }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ThreadMember.ts")]
pub struct ThreadMember {
    pub flags: NotBigU64,
    pub id: Option<String>,
    pub join_timestamp: NotBigU64,
    pub member: Option<Member>,
    // pub presence: Option<Presence>,
    pub user_id: Option<String>,
}

impl From<twilight_model::channel::thread::ThreadMember> for ThreadMember {
    fn from(v: twilight_model::channel::thread::ThreadMember) -> Self {
        Self {
            flags: NotBigU64(v.flags),
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
    pub auto_archive_duration: AutoArchiveDuration,
    pub archive_timestamp: NotBigU64,
    pub invitable: Option<bool>,
    pub locked: bool,
}

impl From<twilight_model::channel::thread::ThreadMetadata> for ThreadMetadata {
    fn from(v: twilight_model::channel::thread::ThreadMetadata) -> Self {
        Self {
            archived: v.archived,
            auto_archive_duration: v.auto_archive_duration.into(),
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
    pub guild_id: String,
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
            guild_id: v
                .guild_id
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
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
    pub allow: String,
    pub deny: String,
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
            allow: v.allow.bits().to_string(),
            deny: v.deny.bits().to_string(),
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
