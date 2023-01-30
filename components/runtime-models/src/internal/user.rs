use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, rename = "IUser")]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/IUser.ts")]
pub struct User {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: String,
    pub id: String,
    pub locale: Option<String>,
    pub username: String,
    pub premium_type: Option<PremiumType>,
    pub public_flags: Option<UserFlags>,
    pub system: Option<bool>,
}

impl From<twilight_model::user::User> for User {
    fn from(v: twilight_model::user::User) -> Self {
        Self {
            avatar: v.avatar.as_ref().map(ToString::to_string),
            bot: v.bot,
            discriminator: v.discriminator().to_string(),
            id: v.id.to_string(),
            locale: v.locale,
            username: v.name,
            premium_type: v.premium_type.map(From::from),
            public_flags: v.public_flags.map(From::from),
            system: v.system,
        }
    }
}
impl From<twilight_model::user::CurrentUser> for User {
    fn from(v: twilight_model::user::CurrentUser) -> Self {
        Self {
            avatar: v.avatar.as_ref().map(ToString::to_string),
            bot: v.bot,
            discriminator: v.discriminator().to_string(),
            id: v.id.to_string(),
            locale: v.locale,
            username: v.name,
            premium_type: v.premium_type.map(From::from),
            public_flags: v.public_flags.map(From::from),
            system: Some(false),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/PremiumType.ts")]
pub enum PremiumType {
    None,
    NitroClassic,
    Nitro,
    NitroBasic,
}

impl From<twilight_model::user::PremiumType> for PremiumType {
    fn from(v: twilight_model::user::PremiumType) -> Self {
        match v {
            twilight_model::user::PremiumType::Nitro => Self::Nitro,
            twilight_model::user::PremiumType::NitroClassic => Self::NitroClassic,
            twilight_model::user::PremiumType::None => Self::None,
            twilight_model::user::PremiumType::NitroBasic => Self::NitroBasic,
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[ts(export, rename = "IUserFlags")]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/IUserFlags.ts")]
pub struct UserFlags {
    pub(crate) staff: bool,                    // Discord Employee
    pub(crate) partner: bool,                  // Partnered Server Owner
    pub(crate) hypesquad: bool,                // HypeSquad Events Coordinator
    pub(crate) bug_hunter_level_1: bool,       // Bug Hunter Level 1
    pub(crate) hypesquad_online_house_1: bool, // House Bravery Member
    pub(crate) hypesquad_online_house_2: bool, // House Brilliance Member
    pub(crate) hypesquad_online_house_3: bool, // House Balance Member
    pub(crate) premium_early_supporter: bool,  // Early Nitro Supporter
    pub(crate) team_pseudo_user: bool,         // User is a team
    pub(crate) bug_hunter_level_2: bool,       // Bug Hunter Level 2
    pub(crate) verified_bot: bool,             // Verified Bot
    pub(crate) verified_developer: bool,       // Early Verified Bot Developer
    pub(crate) certified_moderator: bool,      // Discord Certified Moderator
    pub(crate) bot_http_interactions: bool, // Bot uses only HTTP interactions and is shown in the online member list
}

impl From<twilight_model::user::UserFlags> for UserFlags {
    fn from(uf: twilight_model::user::UserFlags) -> Self {
        Self {
            staff: uf.contains(twilight_model::user::UserFlags::STAFF),
            partner: uf.contains(twilight_model::user::UserFlags::PARTNER),
            hypesquad: uf.contains(twilight_model::user::UserFlags::HYPESQUAD),
            bug_hunter_level_1: uf.contains(twilight_model::user::UserFlags::BUG_HUNTER_LEVEL_1),
            hypesquad_online_house_1: uf
                .contains(twilight_model::user::UserFlags::HYPESQUAD_ONLINE_HOUSE_1),
            hypesquad_online_house_2: uf
                .contains(twilight_model::user::UserFlags::HYPESQUAD_ONLINE_HOUSE_2),
            hypesquad_online_house_3: uf
                .contains(twilight_model::user::UserFlags::HYPESQUAD_ONLINE_HOUSE_3),
            premium_early_supporter: uf
                .contains(twilight_model::user::UserFlags::PREMIUM_EARLY_SUPPORTER),
            team_pseudo_user: uf.contains(twilight_model::user::UserFlags::TEAM_PSEUDO_USER),
            bug_hunter_level_2: uf.contains(twilight_model::user::UserFlags::BUG_HUNTER_LEVEL_2),
            verified_bot: uf.contains(twilight_model::user::UserFlags::VERIFIED_BOT),
            verified_developer: uf.contains(twilight_model::user::UserFlags::VERIFIED_DEVELOPER),
            certified_moderator: uf
                .contains(twilight_model::user::UserFlags::MODERATOR_PROGRAMS_ALUMNI),
            bot_http_interactions: uf
                .contains(twilight_model::user::UserFlags::BOT_HTTP_INTERACTIONS),
        }
    }
}
