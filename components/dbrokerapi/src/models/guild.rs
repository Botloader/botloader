use serde::{Deserialize, Serialize};
use twilight_model::{
    guild::{
        DefaultMessageNotificationLevel, ExplicitContentFilter, MfaLevel, NSFWLevel, Permissions,
        PremiumTier, SystemChannelFlags, VerificationLevel,
    },
    id::{
        marker::{
            ApplicationMarker, ChannelMarker, EmojiMarker, GuildMarker, RoleMarker, UserMarker,
        },
        Id,
    },
    util::Timestamp,
};

/// Represents a cached [`Guild`].
///
/// [`Guild`]: twilight_model::guild::Guild
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BrokerGuild {
    pub afk_channel_id: Option<Id<ChannelMarker>>,
    pub afk_timeout: u64,
    pub application_id: Option<Id<ApplicationMarker>>,
    pub banner: Option<String>,
    pub default_message_notifications: DefaultMessageNotificationLevel,
    pub description: Option<String>,
    pub discovery_splash: Option<String>,
    pub explicit_content_filter: ExplicitContentFilter,
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub id: Id<GuildMarker>,
    pub joined_at: Option<Timestamp>,
    pub large: bool,
    pub max_members: Option<u64>,
    pub max_presences: Option<u64>,
    pub member_count: Option<u64>,
    pub mfa_level: MfaLevel,
    pub name: String,
    pub nsfw_level: NSFWLevel,
    pub owner_id: Id<UserMarker>,
    pub owner: Option<bool>,
    pub permissions: Option<Permissions>,
    pub preferred_locale: String,
    pub premium_progress_bar_enabled: bool,
    pub premium_subscription_count: Option<u64>,
    pub premium_tier: PremiumTier,
    pub rules_channel_id: Option<Id<ChannelMarker>>,
    pub splash: Option<String>,
    pub system_channel_id: Option<Id<ChannelMarker>>,
    pub system_channel_flags: SystemChannelFlags,
    pub unavailable: bool,
    pub vanity_url_code: Option<String>,
    pub verification_level: VerificationLevel,
    pub widget_channel_id: Option<Id<ChannelMarker>>,
    pub widget_enabled: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BrokerEmoji {
    pub animated: bool,
    pub available: bool,
    pub id: Id<EmojiMarker>,
    pub managed: bool,
    pub name: String,
    pub require_colons: bool,
    pub roles: Vec<Id<RoleMarker>>,
    pub user_id: Option<Id<UserMarker>>,
}
