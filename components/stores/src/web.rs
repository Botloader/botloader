use std::{fmt::Debug, time::Duration};

use async_trait::async_trait;
use oauth2::{
    basic::BasicTokenType, CsrfToken, EmptyExtraTokenFields, StandardTokenResponse, TokenResponse,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::CurrentUser,
};

pub type OauthToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum SessionType {
    User,
    ApiKey,
}

#[async_trait]
pub trait SessionStore {
    type Error: std::error::Error + Send + Sync;

    async fn set_user_oatuh_token(
        &self,
        oauth2_token: DiscordOauthToken,
    ) -> Result<DiscordOauthToken, Self::Error>;

    async fn set_oauth_create_session(
        &self,
        oauth2_token: DiscordOauthToken,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Self::Error>;

    async fn create_session(
        &self,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Self::Error>;

    async fn get_oauth_token(
        &self,
        user_id: Id<UserMarker>,
    ) -> Result<DiscordOauthToken, Self::Error>;
    async fn get_session(&self, token: &str) -> Result<Option<Session>, Self::Error>;
    async fn get_all_sessions(&self, user_id: Id<UserMarker>) -> Result<Vec<Session>, Self::Error>;
    async fn del_session(&self, token: &str) -> Result<bool, Self::Error>;
    async fn del_all_sessions(&self, user_id: Id<UserMarker>) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait CsrfStore {
    type Error: std::error::Error;

    async fn generate_csrf_token(&self) -> Result<CsrfToken, Self::Error>;
    async fn check_csrf_token(&self, token: &str) -> Result<bool, Self::Error>;
}

pub fn gen_token() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| thread_rng().gen::<u8>()).collect();
    base64::encode_config(&random_bytes, base64::URL_SAFE_NO_PAD)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub oauth_token: DiscordOauthToken,
    pub token: String,
    pub kind: SessionType,
    pub user: CurrentUser,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscordOauthToken {
    pub user_id: Id<UserMarker>,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expires: chrono::DateTime<chrono::Utc>,
}

impl DiscordOauthToken {
    pub fn new(user_id: Id<UserMarker>, oauth2_token: OauthToken) -> DiscordOauthToken {
        DiscordOauthToken {
            access_token: oauth2_token.access_token().secret().clone(),
            refresh_token: oauth2_token
                .refresh_token()
                .map(|at| at.secret().clone())
                .unwrap_or_default(),
            token_expires: chrono::Utc::now()
                + chrono::Duration::from_std(
                    oauth2_token
                        .expires_in()
                        .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 7)),
                )
                .unwrap(),
            user_id,
        }
    }
}

impl Debug for DiscordOauthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscordOauthToken")
            .field("user", &self.user_id)
            .field("access_token", &"<yoinked>")
            .field("refresh_token", &"<yoinked>")
            .field("token_expires", &self.token_expires)
            .finish()
    }
}
