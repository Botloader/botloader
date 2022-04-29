use crate::util::NotBigU64;
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/PartialMember.ts")]
pub struct PartialMember {
    pub deaf: bool,
    pub joined_at: NotBigU64,
    pub mute: bool,
    pub nick: Option<String>,
    pub premium_since: Option<NotBigU64>,
    pub roles: Vec<String>,
    pub communication_disabled_until: Option<NotBigU64>,
}

impl From<twilight_model::guild::PartialMember> for PartialMember {
    fn from(v: twilight_model::guild::PartialMember) -> Self {
        Self {
            deaf: v.deaf,
            joined_at: NotBigU64(v.joined_at.as_micros() as u64 / 1000),
            mute: v.mute,
            nick: v.nick,
            premium_since: v
                .premium_since
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
            communication_disabled_until: v.communication_disabled_until
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
        }
    }
}
