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

/// Named channel type.
///
/// Channel types botloader doesn't know about (yet) are represented as `Unknown`, scripts can use
/// the accompanying `kindId` field (the raw discord channel type id) to tell them apart.
#[derive(Clone, Debug, Serialize, Deserialize, TS, Copy, PartialEq, Eq, schemars::JsonSchema)]
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
    #[schemars(skip)]
    #[serde(skip_deserializing)]
    Unknown,
}

impl From<twilight_model::channel::ChannelType> for ChannelType {
    fn from(v: twilight_model::channel::ChannelType) -> ChannelType {
        use twilight_model::channel::ChannelType as T;
        match v {
            T::GuildText => Self::Text,
            T::GuildVoice => Self::Voice,
            T::GuildCategory => Self::Category,
            T::GuildAnnouncement => Self::News,
            T::GuildStageVoice => Self::StageVoice,
            T::AnnouncementThread => Self::NewsThread,
            T::PublicThread => Self::PublicThread,
            T::PrivateThread => Self::PrivateThread,
            T::GuildDirectory => Self::GuildDirectory,
            T::GuildForum => Self::Forum,
            T::GuildMedia => Self::Media,
            _ => Self::Unknown,
        }
    }
}

impl TryFrom<ChannelType> for twilight_model::channel::ChannelType {
    type Error = anyhow::Error;

    fn try_from(v: ChannelType) -> Result<Self, Self::Error> {
        Ok(match v {
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
            ChannelType::Unknown => {
                return Err(anyhow::anyhow!(
                    "channel type \"Unknown\" can't be used here, use the numeric channel type id instead"
                ))
            }
        })
    }
}

/// A channel type given either by name (`"Text"`) or by its raw discord channel type id (`0`).
///
/// Used where scripts pass channel types to botloader, so that channel types botloader has no name
/// for yet can still be used.
#[derive(Clone, Debug, Serialize, Deserialize, TS, Copy, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
#[ts(export)]
#[ts(export_to = "bindings/discord/AnyChannelType.ts")]
pub enum AnyChannelType {
    Named(ChannelType),
    Id(u8),
}

impl From<twilight_model::channel::ChannelType> for AnyChannelType {
    fn from(v: twilight_model::channel::ChannelType) -> Self {
        match ChannelType::from(v) {
            ChannelType::Unknown => Self::Id(v.into()),
            named => Self::Named(named),
        }
    }
}

impl TryFrom<AnyChannelType> for twilight_model::channel::ChannelType {
    type Error = anyhow::Error;

    fn try_from(v: AnyChannelType) -> Result<Self, Self::Error> {
        match v {
            AnyChannelType::Named(named) => named.try_into(),
            AnyChannelType::Id(id) => Ok(id.into()),
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

#[cfg(test)]
mod tests {
    use super::{AnyChannelType, ChannelType};
    use twilight_model::channel::ChannelType as TwilightChannelType;

    #[test]
    fn any_channel_type_accepts_names_and_ids() {
        let parsed: Vec<AnyChannelType> = serde_json::from_str(r#"["Forum", 20]"#).unwrap();
        assert_eq!(
            parsed,
            vec![
                AnyChannelType::Named(ChannelType::Forum),
                AnyChannelType::Id(20)
            ]
        );

        let twilight = parsed
            .into_iter()
            .map(TwilightChannelType::try_from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            twilight,
            vec![TwilightChannelType::GuildForum, TwilightChannelType::Unknown(20)]
        );
    }

    #[test]
    fn unknown_name_is_not_accepted_as_input() {
        assert!(serde_json::from_str::<AnyChannelType>(r#""Unknown""#).is_err());
    }

    #[test]
    fn unknown_channel_types_map_to_unknown() {
        assert_eq!(
            ChannelType::from(TwilightChannelType::Unknown(20)),
            ChannelType::Unknown
        );
        assert_eq!(
            serde_json::to_string(&ChannelType::Unknown).unwrap(),
            r#""Unknown""#
        );
        assert_eq!(
            AnyChannelType::from(TwilightChannelType::Unknown(20)),
            AnyChannelType::Id(20)
        );
        assert_eq!(
            AnyChannelType::from(TwilightChannelType::GuildText),
            AnyChannelType::Named(ChannelType::Text)
        );
    }
}
