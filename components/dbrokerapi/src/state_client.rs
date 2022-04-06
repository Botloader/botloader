use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;
use twilight_model::{channel::Channel, guild::Role};

#[derive(Clone)]
pub struct Client {
    server_addr: String,
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error occured for http request: {0}")]
    HTtpError(#[from] reqwest::Error),

    #[error("error error deserializing response: {0}")]
    DesError(#[from] serde_json::Error),

    #[error("another error occured: {0}")]
    Other(String),
}

type ApiResult<T> = Result<T, Error>;

impl Client {
    pub fn new(addr: String) -> Self {
        Self {
            server_addr: addr,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, url: String) -> ApiResult<Option<T>> {
        let resp = self.client.get(url).send().await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(resp.json().await?))
    }

    pub async fn get_guild(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> ApiResult<Option<crate::models::BrokerGuild>> {
        self.get(format!("{}/guilds/{}", self.server_addr, guild_id))
            .await
    }
    pub async fn get_channel(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> ApiResult<Option<Channel>> {
        self.get(format!(
            "{}/guilds/{}/channels/{}",
            self.server_addr, guild_id, channel_id
        ))
        .await
    }

    pub async fn get_channels(&self, guild_id: Id<GuildMarker>) -> ApiResult<Vec<Channel>> {
        self.get(format!("{}/guilds/{}/channels", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }

    pub async fn get_role(
        &self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> ApiResult<Option<Role>> {
        self.get(format!(
            "{}/guilds/{}/roles/{}",
            self.server_addr, guild_id, role_id
        ))
        .await
    }
    pub async fn get_roles(&self, guild_id: Id<GuildMarker>) -> ApiResult<Vec<Role>> {
        self.get(format!("{}/guilds/{}/roles", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }

    pub async fn get_connected_guilds(&self) -> ApiResult<ConnectedGuildsResponse> {
        self.get(format!("{}/connected_guilds", self.server_addr))
            .await
            .map(|inner| inner.unwrap())
    }
}

#[derive(Serialize, Deserialize)]
pub enum ConnectedGuildsResponse {
    NotReady,
    Ready(Vec<Id<GuildMarker>>),
}
