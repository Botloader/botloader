use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::{guild::Permissions, id::Id};

use crate::util::NotBigU64;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
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
            _ => todo!(),
        }
    }
}

impl From<VideoQualityMode> for twilight_model::channel::VideoQualityMode {
    fn from(v: VideoQualityMode) -> Self {
        match v {
            VideoQualityMode::Auto => Self::Auto,
            VideoQualityMode::Full => Self::Full,
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

#[derive(Clone, Debug, Serialize, Deserialize, TS, Copy, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ChannelType.ts")]
pub enum ChannelType {
    Text,
    Voice,
    Category,
    News,
    // Store channels are no longer used
    #[ts(skip)]
    Store,
    StageVoice,
    NewsThread,
    PublicThread,
    PrivateThread,
    GuildDirectory,
    Forum,
    Media,
}

impl TryFrom<twilight_model::channel::ChannelType> for ChannelType {
    type Error = anyhow::Error;

    fn try_from(v: twilight_model::channel::ChannelType) -> Result<ChannelType, anyhow::Error> {
        match v {
            twilight_model::channel::ChannelType::GuildText => Ok(Self::Text),
            twilight_model::channel::ChannelType::GuildVoice => Ok(Self::Voice),
            twilight_model::channel::ChannelType::GuildCategory => Ok(Self::Category),
            twilight_model::channel::ChannelType::GuildAnnouncement => Ok(Self::News),
            twilight_model::channel::ChannelType::GuildStageVoice => Ok(Self::StageVoice),
            twilight_model::channel::ChannelType::AnnouncementThread => Ok(Self::NewsThread),
            twilight_model::channel::ChannelType::PublicThread => Ok(Self::PublicThread),
            twilight_model::channel::ChannelType::PrivateThread => Ok(Self::PrivateThread),
            twilight_model::channel::ChannelType::Group => panic!("unspported channel type: group"),
            twilight_model::channel::ChannelType::Private => {
                panic!("unspported channel type: private")
            }
            twilight_model::channel::ChannelType::GuildDirectory => Ok(Self::GuildDirectory),
            twilight_model::channel::ChannelType::GuildForum => Ok(Self::Forum),
            twilight_model::channel::ChannelType::GuildMedia => Ok(Self::Media),
            other => Err(anyhow::anyhow!(
                "Unimplemented channel type {}",
                u8::from(other)
            )),
        }
    }
}

impl From<ChannelType> for twilight_model::channel::ChannelType {
    fn from(v: ChannelType) -> Self {
        match v {
            ChannelType::Text => Self::GuildText,
            ChannelType::Voice => Self::GuildVoice,
            ChannelType::Category => Self::GuildCategory,
            ChannelType::News => Self::GuildAnnouncement,
            ChannelType::StageVoice => Self::GuildStageVoice,
            ChannelType::NewsThread => Self::AnnouncementThread,
            ChannelType::PublicThread => Self::PublicThread,
            ChannelType::PrivateThread => Self::PrivateThread,
            ChannelType::GuildDirectory => Self::GuildDirectory,
            ChannelType::Forum => Self::GuildForum,
            ChannelType::Store => Self::GuildText,
            ChannelType::Media => Self::GuildMedia,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(
    export_to = "bindings/discord/IPermissionOverwrite.ts",
    rename = "IPermissionOverwrite"
)]
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

impl TryFrom<PermissionOverwrite>
    for twilight_model::channel::permission_overwrite::PermissionOverwrite
{
    type Error = ();

    fn try_from(v: PermissionOverwrite) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Id::new_checked(v.id.parse().map_err(|_| ())?).ok_or(())?,
            allow: Permissions::from_bits_truncate(v.allow_raw.parse().unwrap_or(0)),
            deny: Permissions::from_bits_truncate(v.deny_raw.parse().unwrap_or(0)),
            kind: v.kind.into(),
        })
    }
}

impl TryFrom<PermissionOverwrite>
    for twilight_model::http::permission_overwrite::PermissionOverwrite
{
    type Error = ();

    fn try_from(v: PermissionOverwrite) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Id::new_checked(v.id.parse().map_err(|_| ())?).ok_or(())?,
            allow: Some(Permissions::from_bits_truncate(
                v.allow_raw.parse().unwrap_or(0),
            )),
            deny: Some(Permissions::from_bits_truncate(
                v.deny_raw.parse().unwrap_or(0),
            )),
            kind: v.kind.into(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
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
            _ => todo!(),
        }
    }
}

impl From<PermissionOverwriteType>
    for twilight_model::channel::permission_overwrite::PermissionOverwriteType
{
    fn from(v: PermissionOverwriteType) -> Self {
        match v {
            PermissionOverwriteType::Member => Self::Member,
            PermissionOverwriteType::Role => Self::Role,
        }
    }
}

impl From<PermissionOverwriteType>
    for twilight_model::http::permission_overwrite::PermissionOverwriteType
{
    fn from(v: PermissionOverwriteType) -> Self {
        match v {
            PermissionOverwriteType::Member => Self::Member,
            PermissionOverwriteType::Role => Self::Role,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ForumLayout.ts")]
pub enum ForumLayout {
    GalleryView,
    ListView,
    NotSet,
}

impl From<twilight_model::channel::forum::ForumLayout> for ForumLayout {
    fn from(v: twilight_model::channel::forum::ForumLayout) -> Self {
        match v {
            twilight_model::channel::forum::ForumLayout::GalleryView => Self::GalleryView,
            twilight_model::channel::forum::ForumLayout::ListView => Self::ListView,
            twilight_model::channel::forum::ForumLayout::NotSet => Self::NotSet,
            _ => todo!(),
        }
    }
}

impl From<ForumLayout> for twilight_model::channel::forum::ForumLayout {
    fn from(v: ForumLayout) -> Self {
        match v {
            ForumLayout::GalleryView => Self::GalleryView,
            ForumLayout::ListView => Self::ListView,
            ForumLayout::NotSet => Self::NotSet,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ForumSortOrder.ts")]
pub enum ForumSortOrder {
    CreationDate,
    LatestActivity,
}

impl From<twilight_model::channel::forum::ForumSortOrder> for ForumSortOrder {
    fn from(v: twilight_model::channel::forum::ForumSortOrder) -> Self {
        match v {
            twilight_model::channel::forum::ForumSortOrder::CreationDate => Self::CreationDate,
            twilight_model::channel::forum::ForumSortOrder::LatestActivity => Self::LatestActivity,
            _ => todo!(),
        }
    }
}

impl From<ForumSortOrder> for twilight_model::channel::forum::ForumSortOrder {
    fn from(v: ForumSortOrder) -> Self {
        match v {
            ForumSortOrder::CreationDate => Self::CreationDate,
            ForumSortOrder::LatestActivity => Self::LatestActivity,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/ChannelFlags.ts")]
pub struct ChannelFlags {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pinned: Option<bool>, //  1 << 0	this thread is pinned to the top of its parent GUILD_FORUM or GUILD_MEDIA channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub require_tag: Option<bool>, // 1 << 4    whether a tag is required to be specified when creating a thread in a GUILD_FORUM or a GUILD_MEDIA channel. Tags are specified in the applied_tags field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hide_media_download_options: Option<bool>, // 1 << 15   when set hides the embedded media download options. Available only for media channels
}

impl From<twilight_model::channel::ChannelFlags> for ChannelFlags {
    fn from(v: twilight_model::channel::ChannelFlags) -> Self {
        use twilight_model::channel::ChannelFlags as TwilightChannelFlags;

        Self {
            pinned: Some(v.contains(TwilightChannelFlags::PINNED)),
            require_tag: Some(v.contains(TwilightChannelFlags::REQUIRE_TAG)),
            hide_media_download_options: Some(v.contains(TwilightChannelFlags::HIDE_MEDIA_DOWNLOAD_OPTIONS)),
        }
    }
}

impl From<ChannelFlags> for twilight_model::channel::ChannelFlags {
    fn from(v: ChannelFlags) -> Self {
        let mut out = Self::empty();

        if matches!(v.pinned, Some(true)) {
            out |= Self::PINNED;
        }
        
        if matches!(v.require_tag, Some(true)) {
            out |= Self::REQUIRE_TAG;
        }
        
        if matches!(v.hide_media_download_options, Some(true)) {
            out |= Self::HIDE_MEDIA_DOWNLOAD_OPTIONS;
        }

        out
    }
}
