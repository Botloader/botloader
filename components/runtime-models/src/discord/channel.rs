use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::util::NotBigU64;

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
#[ts(export_to = "bindings/discord/ThreadMetadata.ts")]
#[serde(rename_all = "camelCase")]
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

impl From<ChannelType> for twilight_model::channel::ChannelType {
    fn from(v: ChannelType) -> Self {
        match v {
            ChannelType::Text => twilight_model::channel::ChannelType::GuildText,
            ChannelType::Voice => twilight_model::channel::ChannelType::GuildVoice,
            ChannelType::Category => twilight_model::channel::ChannelType::GuildCategory,
            ChannelType::News => twilight_model::channel::ChannelType::GuildNews,
            ChannelType::Store => twilight_model::channel::ChannelType::GuildStore,
            ChannelType::StageVoice => twilight_model::channel::ChannelType::GuildStageVoice,
            ChannelType::NewsThread => twilight_model::channel::ChannelType::GuildNewsThread,
            ChannelType::PublicThread => twilight_model::channel::ChannelType::GuildPublicThread,
            ChannelType::PrivateThread => twilight_model::channel::ChannelType::GuildPrivateThread,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/PermissionOverwrite.ts")]
#[serde(rename_all = "camelCase")]
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
            id: v.id.to_string(),
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
            twilight_model::channel::permission_overwrite::PermissionOverwriteType::Member => {
                Self::Member
            }
            twilight_model::channel::permission_overwrite::PermissionOverwriteType::Role => {
                Self::Role
            }
        }
    }
}
