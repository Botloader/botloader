use std::fmt::Write;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use twilight_model::guild::Member;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::voice::VoiceState;
use twilight_model::{channel::Channel, guild::Role};

use crate::models::BrokerEmoji;

#[derive(Clone)]
pub struct Client {
    server_addr: String,
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error occurred for http request: {0}")]
    HTtpError(#[from] reqwest::Error),

    #[error("error error deserializing response: {0}")]
    DesError(#[from] serde_json::Error),

    #[error("Server error occurred: {0}")]
    Server(String),
}

type ApiResult<T> = Result<T, Error>;

impl Client {
    pub fn new(addr: String) -> Self {
        Self {
            server_addr: addr,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_option<T: DeserializeOwned>(&self, url: String) -> ApiResult<Option<T>> {
        let resp = self.client.get(url).send().await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        } else if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or("Unknown error".to_owned());
            return Err(Error::Server(text));
        }

        Ok(Some(resp.json().await?))
    }

    pub async fn get_guild(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ApiResult<Option<crate::models::BrokerGuild>> {
        self.get_option(format!("{}/guilds/{}", self.server_addr, guild_id))
            .await
    }

    pub async fn get_guild_voice_states(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ApiResult<Vec<VoiceState>> {
        self.get_option(format!(
            "{}/guilds/{}/voice_states",
            self.server_addr, guild_id
        ))
        .await
        .map(|v| v.unwrap_or_default())
    }

    pub async fn get_guild_emojis(&self, guild_id: Id<GuildMarker>) -> ApiResult<Vec<BrokerEmoji>> {
        self.get_option(format!("{}/guilds/{}/emojis", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }

    pub async fn get_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> ApiResult<Option<Channel>> {
        self.get_option(format!(
            "{}/guilds/{}/channels/{}",
            self.server_addr, guild_id, channel_id
        ))
        .await
    }

    pub async fn get_channels(&self, guild_id: Id<GuildMarker>) -> ApiResult<Vec<Channel>> {
        self.get_option(format!("{}/guilds/{}/channels", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }

    pub async fn get_role(
        &self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> ApiResult<Option<Role>> {
        self.get_option(format!(
            "{}/guilds/{}/roles/{}",
            self.server_addr, guild_id, role_id
        ))
        .await
    }
    pub async fn get_roles(&self, guild_id: Id<GuildMarker>) -> ApiResult<Vec<Role>> {
        self.get_option(format!("{}/guilds/{}/roles", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }

    pub async fn get_connected_guilds(&self) -> ApiResult<ConnectedGuildsResponse> {
        self.get_option(format!("{}/connected_guilds", self.server_addr))
            .await
            .map(|inner| inner.unwrap())
    }

    pub async fn get_guild_members(
        &self,
        guild_id: Id<GuildMarker>,
        users_ids: Vec<Id<UserMarker>>,
    ) -> ApiResult<Vec<Member>> {
        let mut user_query = String::with_capacity(30 * users_ids.len());
        for (i, user_id) in users_ids.iter().enumerate() {
            if i != 0 {
                user_query.write_char('&').unwrap();
            }

            user_query.write_str("user_id=").unwrap();
            user_query.write_fmt(format_args!("{user_id}")).unwrap();
        }

        self.get_option(format!(
            "{}/guilds/{}/members?{}",
            self.server_addr, guild_id, user_query
        ))
        .await
        .map(|inner| inner.unwrap())
    }
}

#[derive(Serialize, Deserialize)]
pub enum ConnectedGuildsResponse {
    NotReady,
    Ready(Vec<Id<GuildMarker>>),
}
