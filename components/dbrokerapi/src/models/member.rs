use serde::Deserialize;
use twilight_model::{
    datetime::Timestamp,
    id::{
        marker::{GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};

#[derive(Clone, Debug, Deserialize)]
pub struct BrokerMember {
    pub avatar: Option<String>,
    pub communication_disabled_until: Option<Timestamp>,
    pub deaf: Option<bool>,
    pub guild_id: Id<GuildMarker>,
    pub joined_at: Timestamp,
    pub mute: Option<bool>,
    pub nick: Option<String>,
    pub pending: bool,
    pub premium_since: Option<Timestamp>,
    pub roles: Vec<Id<RoleMarker>>,
    pub user_id: Id<UserMarker>,
}
