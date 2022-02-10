use serde::Deserialize;
use twilight_model::{
    datetime::Timestamp,
    id::{GuildId, RoleId, UserId},
};

#[derive(Clone, Debug, Deserialize)]
pub struct BrokerMember {
    pub avatar: Option<String>,
    pub communication_disabled_until: Option<Timestamp>,
    pub deaf: Option<bool>,
    pub guild_id: GuildId,
    pub joined_at: Timestamp,
    pub mute: Option<bool>,
    pub nick: Option<String>,
    pub pending: bool,
    pub premium_since: Option<Timestamp>,
    pub roles: Vec<RoleId>,
    pub user_id: UserId,
}
