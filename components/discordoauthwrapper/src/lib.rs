use std::{
    fmt::{Debug, Display},
    future::Future,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use entities::discord_oauth_tokens;
use oauth2::{
    basic::BasicTokenType, reqwest::async_http_client, EmptyExtraTokenFields,
    StandardTokenResponse, TokenResponse,
};
use sea_orm::{
    sea_query::OnConflict, sqlx::types::chrono, ActiveValue, DatabaseConnection, EntityTrait,
    Iterable,
};
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::{CurrentUser, CurrentUserGuild},
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

mod cache;
mod twilight_api_provider;
pub use cache::ClientCache;
pub use twilight_api_provider::TwilightApiProvider;

struct ApiClientInner<T, TU> {
    user_id: Id<UserMarker>,
    api_provider: T,
    token_refresher: TU,
    db: DatabaseConnection,

    // if the refresh token is no longer valid
    broken: AtomicBool,
}

pub struct DiscordOauthApiClient<T, TU> {
    inner: Arc<ApiClientInner<T, TU>>,
}

impl<T, TU> Clone for DiscordOauthApiClient<T, TU> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TU> DiscordOauthApiClient<TwilightApiProvider, TU>
where
    TU: TokenRefresher,
{
    pub fn new_twilight(
        user_id: Id<UserMarker>,
        bearer_token: String,
        token_refresher: TU,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            inner: Arc::new(ApiClientInner {
                api_provider: TwilightApiProvider::new(twilight_http::Client::new(format!(
                    "Bearer {bearer_token}",
                ))),
                user_id,
                token_refresher,
                db,
                broken: AtomicBool::new(false),
            }),
        }
    }
}

impl<T, TU> DiscordOauthApiClient<T, TU>
where
    T: DiscordOauthApiProvider + 'static,
    TU: TokenRefresher + 'static,
    T::OtherError: Debug + Display + Send + Sync + 'static,
{
    pub fn new(
        user_id: Id<UserMarker>,
        api_provider: T,
        token_refresher: TU,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            inner: Arc::new(ApiClientInner {
                user_id,
                api_provider,
                token_refresher,
                db,
                broken: AtomicBool::new(false),
            }),
        }
    }

    pub async fn current_user(&self) -> Result<CurrentUser, BoxError> {
        self.run_api_check_err(|| self.inner.api_provider.get_current_user())
            .await
    }

    pub async fn current_user_guilds(&self) -> Result<Vec<CurrentUserGuild>, BoxError> {
        self.run_api_check_err(|| self.inner.api_provider.get_user_guilds())
            .await
    }

    // runs the provided closure, refreshing the token if needed
    async fn run_api_check_err<F, FRT, Fut>(&self, f: F) -> Result<FRT, BoxError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<FRT, ApiProviderError<T::OtherError>>>,
    {
        let mut updated_token = false;
        loop {
            match f().await {
                Ok(v) => return Ok(v),
                Err(ApiProviderError::InvalidToken) => {
                    if updated_token {
                        self.inner
                            .broken
                            .store(true, std::sync::atomic::Ordering::SeqCst);

                        return Err(anyhow::anyhow!("invalid token twice").into());
                    }

                    if let Err(err) = self.update_token().await {
                        self.inner
                            .broken
                            .store(true, std::sync::atomic::Ordering::SeqCst);

                        return Err(err);
                    }
                    updated_token = true;
                }
                Err(ApiProviderError::Ratelimit(dur)) => {
                    tokio::time::sleep(dur).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub async fn update_token(&self) -> Result<(), BoxError> {
        let Some(current) = discord_oauth_tokens::Entity::find_by_id(self.inner.user_id)
            .one(&self.inner.db)
            .await?
        else {
            return Err("Could not find auth token entry".into());
        };

        let oauth2_token = self
            .inner
            .token_refresher
            .update_token(current.discord_bearer_token)
            .await?;

        let new_token_entry = discord_oauth_tokens::ActiveModel {
            user_id: ActiveValue::Set(self.inner.user_id.into()),
            discord_bearer_token: ActiveValue::Set(oauth2_token.access_token().secret().clone()),
            discord_refresh_token: ActiveValue::Set(
                oauth2_token
                    .refresh_token()
                    .map(|at| at.secret().clone())
                    .unwrap_or_default(),
            ),
            discord_token_expires_at: ActiveValue::Set(
                (chrono::Utc::now()
                    + oauth2_token
                        .expires_in()
                        .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 7)))
                .into(),
            ),
        };

        discord_oauth_tokens::Entity::insert(new_token_entry)
            .on_conflict(
                OnConflict::column(discord_oauth_tokens::Column::UserId)
                    .update_columns(discord_oauth_tokens::Column::iter())
                    .to_owned(),
            )
            .exec(&self.inner.db)
            .await?;

        self.inner
            .api_provider
            .update_token(oauth2_token.access_token().secret().clone())
            .await;

        Ok(())
    }

    pub fn is_broken(&self) -> bool {
        self.inner.broken.load(std::sync::atomic::Ordering::SeqCst)
    }
}

type OauthTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[async_trait::async_trait]
pub trait TokenRefresher {
    async fn update_token(&self, refresh_token: String) -> Result<OauthTokenResponse, BoxError>;
}

#[derive(Debug)]
pub enum ApiProviderError<T> {
    InvalidToken,
    Ratelimit(Duration),
    Other(T),
}

impl<T: std::fmt::Debug + Display> std::error::Error for ApiProviderError<T> {}

impl<T: std::fmt::Debug + Display> Display for ApiProviderError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => f.write_str("invalid token"),
            Self::Ratelimit(dur) => f.write_fmt(format_args!("ratelimited: {dur:?}")),
            Self::Other(inner) => f.write_fmt(format_args!("{inner}")),
        }
    }
}

#[async_trait::async_trait]
pub trait DiscordOauthApiProvider {
    type OtherError;

    async fn get_current_user(&self) -> Result<CurrentUser, ApiProviderError<Self::OtherError>>;
    async fn get_user_guilds(
        &self,
    ) -> Result<Vec<CurrentUserGuild>, ApiProviderError<Self::OtherError>>;
    async fn update_token(&self, access_token: String);
}

#[async_trait::async_trait]
impl TokenRefresher for oauth2::basic::BasicClient {
    async fn update_token(&self, refresh_token: String) -> Result<OauthTokenResponse, BoxError> {
        let token = oauth2::RefreshToken::new(refresh_token);

        Ok(self
            .exchange_refresh_token(&token)
            .request_async(async_http_client)
            .await?)
    }
}
