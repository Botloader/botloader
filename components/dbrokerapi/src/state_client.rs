use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use twilight_model::{
    channel::GuildChannel,
    guild::Role,
    id::{ChannelId, GuildId, RoleId},
};

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
        guild_id: GuildId,
    ) -> ApiResult<Option<crate::models::BrokerGuild>> {
        self.get(format!("{}/guilds/{}", self.server_addr, guild_id))
            .await
    }
    pub async fn get_channel(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> ApiResult<Option<GuildChannel>> {
        self.get(format!(
            "{}/guilds/{}/channels/{}",
            self.server_addr, guild_id, channel_id
        ))
        .await
    }

    pub async fn get_channels(&self, guild_id: GuildId) -> ApiResult<Vec<GuildChannel>> {
        self.get(format!("{}/guilds/{}/channels", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }

    pub async fn get_role(&self, guild_id: GuildId, role_id: RoleId) -> ApiResult<Option<Role>> {
        self.get(format!(
            "{}/guilds/{}/roles/{}",
            self.server_addr, guild_id, role_id
        ))
        .await
    }
    pub async fn get_roles(&self, guild_id: GuildId) -> ApiResult<Vec<Role>> {
        self.get(format!("{}/guilds/{}/roles", self.server_addr, guild_id))
            .await
            .map(|v| v.unwrap_or_default())
    }
}
