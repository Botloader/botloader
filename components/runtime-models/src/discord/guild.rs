use dbrokerapi::models::BrokerGuild;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_cache_inmemory::model::CachedGuild;
use twilight_model::guild::{
    DefaultMessageNotificationLevel as TwilightDefaultMessageNotificationLevel,
    ExplicitContentFilter as TwilightExplicitContentFilter, MfaLevel as TwilightMfaLevel,
    NSFWLevel as TwilightNSFWLevel, PremiumTier as TwilightPremiumTier,
    VerificationLevel as TwilightVerificationLevel,
};

use crate::util::NotBigU64;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/Guild.ts")]
pub struct Guild {
    pub(crate) afk_channel_id: Option<String>,
    pub(crate) afk_timeout: NotBigU64,
    pub(crate) application_id: Option<String>,
    pub(crate) banner: Option<String>,
    pub(crate) default_message_notifications: DefaultMessageNotificationLevel,
    pub(crate) description: Option<String>,
    pub(crate) discovery_splash: Option<String>,
    pub(crate) explicit_content_filter: ExplicitContentFilter,
    pub(crate) features: Vec<String>,
    pub(crate) icon: Option<String>,
    pub(crate) id: String,
    pub(crate) joined_at: Option<NotBigU64>,
    pub(crate) large: bool,
    pub(crate) max_members: Option<NotBigU64>,
    pub(crate) max_presences: Option<NotBigU64>,
    pub(crate) member_count: Option<NotBigU64>,
    pub(crate) mfa_level: MfaLevel,
    pub(crate) name: String,
    pub(crate) nsfw_level: NsfwLevel,
    pub(crate) owner_id: String,
    pub(crate) preferred_locale: String,
    pub(crate) premium_subscription_count: Option<NotBigU64>,
    pub(crate) premium_tier: PremiumTier,
    pub(crate) rules_channel_id: Option<String>,
    pub(crate) splash: Option<String>,
    pub(crate) system_channel_id: Option<String>,
    pub(crate) unavailable: bool,
    pub(crate) vanity_url_code: Option<String>,
    pub(crate) verification_level: VerificationLevel,
    pub(crate) widget_channel_id: Option<String>,
    pub(crate) widget_enabled: Option<bool>,
    // TODO: how should we represent this? bitflags or something more user accessible?
    // pub(crate) system_channel_flags: SystemChannelFlags,
}

impl From<&CachedGuild> for Guild {
    fn from(v: &CachedGuild) -> Self {
        Self {
            afk_channel_id: v.afk_channel_id().as_ref().map(ToString::to_string),
            afk_timeout: NotBigU64(v.afk_timeout()),
            application_id: v.application_id().as_ref().map(ToString::to_string),
            banner: v.banner().map(ToString::to_string),
            default_message_notifications: v.default_message_notifications().into(),
            description: v.description().map(ToString::to_string),
            discovery_splash: v.discovery_splash().map(ToString::to_string),
            explicit_content_filter: v.explicit_content_filter().into(),
            features: v.features().map(ToString::to_string).collect(),
            icon: v.icon().map(ToString::to_string),
            id: v.id().to_string(),
            joined_at: v
                .joined_at()
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            large: v.large(),
            max_members: v.max_members().map(NotBigU64),
            max_presences: v.max_presences().map(NotBigU64),
            member_count: v.member_count().map(NotBigU64),
            mfa_level: v.mfa_level().into(),
            name: v.name().to_string(),
            nsfw_level: v.nsfw_level().into(),
            owner_id: v.owner_id().to_string(),
            preferred_locale: v.preferred_locale().to_string(),
            premium_subscription_count: v.premium_subscription_count().map(NotBigU64),
            premium_tier: v.premium_tier().into(),
            rules_channel_id: v.rules_channel_id().as_ref().map(ToString::to_string),
            splash: v.splash().map(ToString::to_string),
            system_channel_id: v.system_channel_id().as_ref().map(ToString::to_string),
            unavailable: v.unavailable(),
            vanity_url_code: v.vanity_url_code().map(ToString::to_string),
            verification_level: v.verification_level().into(),
            widget_channel_id: v.widget_channel_id().as_ref().map(ToString::to_string),
            widget_enabled: v.widget_enabled(),
        }
    }
}

impl From<BrokerGuild> for Guild {
    fn from(v: BrokerGuild) -> Self {
        Self {
            afk_channel_id: v.afk_channel_id.as_ref().map(ToString::to_string),
            afk_timeout: NotBigU64(v.afk_timeout),
            application_id: v.application_id.as_ref().map(ToString::to_string),
            banner: v.banner,
            default_message_notifications: v.default_message_notifications.into(),
            description: v.description,
            discovery_splash: v.discovery_splash,
            explicit_content_filter: v.explicit_content_filter.into(),
            features: v.features,
            icon: v.icon,
            id: v.id.to_string(),
            joined_at: v
                .joined_at
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            large: v.large,
            max_members: v.max_members.map(NotBigU64),
            max_presences: v.max_presences.map(NotBigU64),
            member_count: v.member_count.map(NotBigU64),
            mfa_level: v.mfa_level.into(),
            name: v.name.to_string(),
            nsfw_level: v.nsfw_level.into(),
            owner_id: v.owner_id.to_string(),
            preferred_locale: v.preferred_locale.to_string(),
            premium_subscription_count: v.premium_subscription_count.map(NotBigU64),
            premium_tier: v.premium_tier.into(),
            rules_channel_id: v.rules_channel_id.as_ref().map(ToString::to_string),
            splash: v.splash,
            system_channel_id: v.system_channel_id.as_ref().map(ToString::to_string),
            unavailable: v.unavailable,
            vanity_url_code: v.vanity_url_code,
            verification_level: v.verification_level.into(),
            widget_channel_id: v.widget_channel_id.as_ref().map(ToString::to_string),
            widget_enabled: v.widget_enabled,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/DefaultMessageNotificationLevel.ts")]
pub enum DefaultMessageNotificationLevel {
    All,
    Mentions,
}

impl From<TwilightDefaultMessageNotificationLevel> for DefaultMessageNotificationLevel {
    fn from(v: TwilightDefaultMessageNotificationLevel) -> Self {
        match v {
            TwilightDefaultMessageNotificationLevel::All => Self::All,
            TwilightDefaultMessageNotificationLevel::Mentions => Self::Mentions,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/ExplicitContentFilter.ts")]
pub enum ExplicitContentFilter {
    None,
    MembersWithoutRole,
    AllMembers,
}

impl From<TwilightExplicitContentFilter> for ExplicitContentFilter {
    fn from(v: TwilightExplicitContentFilter) -> Self {
        match v {
            TwilightExplicitContentFilter::None => Self::None,
            TwilightExplicitContentFilter::MembersWithoutRole => Self::MembersWithoutRole,
            TwilightExplicitContentFilter::AllMembers => Self::AllMembers,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/MfaLevel.ts")]
pub enum MfaLevel {
    None,
    Elevated,
}

impl From<TwilightMfaLevel> for MfaLevel {
    fn from(v: TwilightMfaLevel) -> Self {
        match v {
            TwilightMfaLevel::None => Self::None,
            TwilightMfaLevel::Elevated => Self::Elevated,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/NsfwLevel.ts")]
pub enum NsfwLevel {
    Default,
    Explicit,
    Safe,
    AgeRestricted,
}

impl From<TwilightNSFWLevel> for NsfwLevel {
    fn from(v: TwilightNSFWLevel) -> Self {
        match v {
            TwilightNSFWLevel::Default => Self::Default,
            TwilightNSFWLevel::Explicit => Self::Explicit,
            TwilightNSFWLevel::Safe => Self::Safe,
            TwilightNSFWLevel::AgeRestricted => Self::AgeRestricted,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/PremiumTier.ts")]
pub enum PremiumTier {
    None,
    Tier1,
    Tier2,
    Tier3,
}

impl From<TwilightPremiumTier> for PremiumTier {
    fn from(v: TwilightPremiumTier) -> Self {
        match v {
            TwilightPremiumTier::None => Self::None,
            TwilightPremiumTier::Tier1 => Self::Tier1,
            TwilightPremiumTier::Tier2 => Self::Tier2,
            TwilightPremiumTier::Tier3 => Self::Tier3,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/VerificationLevel.ts")]
pub enum VerificationLevel {
    None,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl From<TwilightVerificationLevel> for VerificationLevel {
    fn from(v: TwilightVerificationLevel) -> Self {
        match v {
            TwilightVerificationLevel::None => Self::None,
            TwilightVerificationLevel::Low => Self::Low,
            TwilightVerificationLevel::Medium => Self::Medium,
            TwilightVerificationLevel::High => Self::High,
            TwilightVerificationLevel::VeryHigh => Self::VeryHigh,
        }
    }
}
