use crate::{internal::user::User, util::NotBigU64};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::{
    id::{
        marker::{ChannelMarker, RoleMarker},
        Id,
    },
    util::Timestamp,
};

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/UpdateGuildMemberFields.ts")]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuildMemberFields {
    #[ts(optional)]
    #[ts(type = "string|null")]
    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    pub channel_id: Option<Option<Id<ChannelMarker>>>,

    #[ts(optional)]
    pub deaf: Option<bool>,

    #[ts(optional)]
    pub mute: Option<bool>,

    #[ts(optional)]
    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    pub nick: Option<Option<String>>,

    #[ts(optional)]
    #[ts(type = "string[]")]
    pub roles: Option<Vec<Id<RoleMarker>>>,

    #[ts(optional)]
    #[ts(type = "number|null")]
    #[serde(
        deserialize_with = "crate::deserialize_undefined_null_optional_field",
        default
    )]
    pub communication_disabled_until: Option<Option<NotBigU64>>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IMember")]
#[ts(export_to = "bindings/internal/Member.ts")]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub deaf: bool,
    pub joined_at: NotBigU64,
    pub mute: bool,
    pub nick: Option<String>,
    pub pending: bool,
    pub premium_since: Option<NotBigU64>,
    pub roles: Vec<String>,
    pub communication_disabled_until: Option<NotBigU64>,
    pub user: User,
}

impl From<twilight_model::guild::Member> for Member {
    fn from(v: twilight_model::guild::Member) -> Self {
        Self {
            deaf: v.deaf,
            joined_at: NotBigU64(
                v.joined_at
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros() as u64
                    / 1000,
            ),
            mute: v.mute,
            nick: v.nick,
            pending: v.pending,
            premium_since: v
                .premium_since
                .map(|v| NotBigU64(v.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
            communication_disabled_until: v
                .communication_disabled_until
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            user: v.user.into(),
        }
    }
}

impl Member {
    pub fn from_cache(user: User, member: twilight_cache_inmemory::model::CachedMember) -> Self {
        Self {
            user,
            deaf: member.deaf().unwrap_or_default(),
            joined_at: NotBigU64(
                member
                    .joined_at()
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros() as u64
                    / 1000,
            ),
            mute: member.mute().unwrap_or_default(),
            nick: member.nick().map(ToString::to_string),
            premium_since: member
                .premium_since()
                .map(|v| NotBigU64(v.as_micros() as u64 / 1000)),
            roles: member.roles().iter().map(ToString::to_string).collect(),
            communication_disabled_until: member
                .communication_disabled_until()
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            pending: false,
        }
    }

    pub fn from_partial(partial: twilight_model::guild::PartialMember) -> Self {
        Self {
            joined_at: NotBigU64(
                partial
                    .joined_at
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros() as u64
                    / 1000,
            ),
            nick: partial.nick,
            premium_since: partial
                .premium_since
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            roles: partial.roles.iter().map(ToString::to_string).collect(),
            communication_disabled_until: partial
                .communication_disabled_until
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            user: partial.user.unwrap().into(),
            deaf: partial.deaf,
            mute: partial.mute,
            // From what i can see in the docs, pending is only omitted in events where it's always false, as they
            // can't be pending to trigger that event
            // so this should be sufficient for now.
            pending: false,
        }
    }
}

impl From<twilight_model::gateway::payload::incoming::MemberUpdate> for Member {
    fn from(v: twilight_model::gateway::payload::incoming::MemberUpdate) -> Self {
        Self {
            deaf: v.deaf.unwrap_or_default(),
            joined_at: NotBigU64(
                v.joined_at
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros() as u64
                    / 1000,
            ),
            mute: v.mute.unwrap_or_default(),
            nick: v.nick,
            pending: v.pending,
            premium_since: v
                .premium_since
                .map(|v| NotBigU64(v.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
            communication_disabled_until: v
                .communication_disabled_until
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            user: v.user.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IBan")]
#[ts(export_to = "bindings/internal/Ban.ts")]
#[serde(rename_all = "camelCase")]
pub struct Ban {
    reason: Option<String>,
    user: User,
}

impl From<twilight_model::guild::Ban> for Ban {
    fn from(v: twilight_model::guild::Ban) -> Self {
        Self {
            reason: v.reason,
            user: v.user.into(),
        }
    }
}
