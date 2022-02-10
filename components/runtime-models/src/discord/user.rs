use serde::Serialize;
use ts_rs::TS;

use crate::util::NotBigU64;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/User.ts")]
pub struct User {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: u16,
    pub email: Option<String>,
    pub id: String,
    pub locale: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub username: String,
    pub premium_type: Option<PremiumType>,
    pub public_flags: Option<NotBigU64>,
    pub system: Option<bool>,
    pub verified: Option<bool>,
}

impl From<twilight_model::user::User> for User {
    fn from(v: twilight_model::user::User) -> Self {
        Self {
            avatar: v.avatar,
            bot: v.bot,
            discriminator: v.discriminator,
            email: v.email,
            id: v.id.to_string(),
            locale: v.locale,
            mfa_enabled: v.mfa_enabled,
            username: v.name,
            premium_type: v.premium_type.map(From::from),
            public_flags: v.public_flags.map(|e| NotBigU64(e.bits())),
            system: v.system,
            verified: v.verified,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/PremiumType.ts")]
pub enum PremiumType {
    None,
    NitroClassic,
    Nitro,
}

impl From<twilight_model::user::PremiumType> for PremiumType {
    fn from(v: twilight_model::user::PremiumType) -> Self {
        match v {
            twilight_model::user::PremiumType::Nitro => Self::Nitro,
            twilight_model::user::PremiumType::NitroClassic => Self::NitroClassic,
            twilight_model::user::PremiumType::None => Self::None,
        }
    }
}
