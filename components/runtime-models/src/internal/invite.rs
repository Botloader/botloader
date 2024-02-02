use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::id::{
    marker::{ApplicationMarker, UserMarker},
    Id,
};

use crate::{
    discord::invite::{InviteChannel, InviteGuild, InviteTargetType},
    util::NotBigU64,
};

use super::user::User;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IInvite")]
#[ts(export_to = "bindings/internal/IInvite.ts")]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate_member_count: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate_presence_count: Option<NotBigU64>,
    pub channel: Option<InviteChannel>,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild: Option<InviteGuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inviter: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<NotBigU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<InviteTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_user: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<NotBigU64>,
}

impl TryFrom<twilight_model::guild::invite::Invite> for Invite {
    type Error = anyhow::Error;

    fn try_from(value: twilight_model::guild::invite::Invite) -> Result<Self, Self::Error> {
        Ok(Self {
            approximate_member_count: value.approximate_member_count.map(Into::into),
            approximate_presence_count: value.approximate_presence_count.map(Into::into),
            channel: value.channel.map(Into::into),
            code: value.code,
            created_at: value
                .created_at
                .map(|v| ((v.as_micros() / 1000) as u64).into()),
            expires_at: value
                .expires_at
                .map(|v| ((v.as_micros() / 1000) as u64).into()),
            // expires_at: value.expires_at,
            guild: value.guild.map(Into::into),
            inviter: value.inviter.map(Into::into),
            max_age: value.max_age.map(Into::into),
            max_uses: value.max_uses.map(Into::into),
            target_type: if let Some(t) = value.target_type {
                Some(t.try_into()?)
            } else {
                None
            },
            target_user: value.target_user.map(Into::into),
            temporary: value.temporary,
            uses: value.uses.map(Into::into),
        })
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, rename = "ICreateInviteFields")]
#[ts(export_to = "bindings/internal/ICreateInviteFields.ts")]

pub struct CreateInviteFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string")]
    pub target_application_id: Option<Id<ApplicationMarker>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string")]
    pub target_user_id: Option<Id<UserMarker>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<InviteTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
}
