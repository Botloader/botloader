use std::{
    fmt::{Debug, Display},
    future::Future,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use oauth2::reqwest::async_http_client;
use stores::web::{DiscordOauthToken, SessionStore};
use twilight_model::{
    id::UserId,
    user::{CurrentUser, CurrentUserGuild},
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

mod cache;
mod twilight_api_provider;
pub use cache::ClientCache;
pub use twilight_api_provider::TwilightApiProvider;

struct ApiClientInner<T, TU, ST> {
    user_id: UserId,
    api_provider: T,
    token_refresher: TU,
    session_store: ST,

    // if the refresh token is no longer valid
    broken: AtomicBool,
}

pub struct DiscordOauthApiClient<T, TU, ST> {
    inner: Arc<ApiClientInner<T, TU, ST>>,
}

impl<T, TU, ST> Clone for DiscordOauthApiClient<T, TU, ST> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TU, ST> DiscordOauthApiClient<TwilightApiProvider, TU, ST>
where
    TU: TokenRefresher,
    ST: SessionStore,
{
    pub fn new_twilight(
        user_id: UserId,
        bearer_token: String,
        token_refresher: TU,
        session_store: ST,
    ) -> Self {
        Self {
            inner: Arc::new(ApiClientInner {
                api_provider: TwilightApiProvider::new(twilight_http::Client::new(format!(
                    "Bearer {}",
                    bearer_token
                ))),
                user_id,
                token_refresher,
                session_store,
                broken: AtomicBool::new(false),
            }),
        }
    }
}

impl<T, TU, ST> DiscordOauthApiClient<T, TU, ST>
where
    T: DiscordOauthApiProvider + 'static,
    TU: TokenRefresher + 'static,
    ST: SessionStore + 'static,
    T::OtherError: Debug + Display + Send + Sync + 'static,
{
    pub fn new(user_id: UserId, api_provider: T, token_refresher: TU, session_store: ST) -> Self {
        Self {
            inner: Arc::new(ApiClientInner {
                user_id,
                api_provider,
                token_refresher,
                session_store,
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
        let current = self
            .inner
            .session_store
            .get_oauth_token(self.inner.user_id)
            .await?;

        let new_token = DiscordOauthToken::new(
            self.inner.user_id,
            self.inner.token_refresher.update_token(current).await?,
        );

        let access_token = new_token.access_token.clone();
        self.inner
            .session_store
            .set_user_oatuh_token(new_token)
            .await?;

        self.inner.api_provider.update_token(access_token).await;

        Ok(())
    }

    pub fn is_broken(&self) -> bool {
        self.inner.broken.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
pub trait TokenRefresher {
    async fn update_token(
        &self,
        token: DiscordOauthToken,
    ) -> Result<stores::web::OauthToken, BoxError>;
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
            Self::Ratelimit(dur) => f.write_fmt(format_args!("ratelimited: {:?}", dur)),
            Self::Other(inner) => f.write_fmt(format_args!("{}", inner)),
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
    async fn update_token(
        &self,
        token: DiscordOauthToken,
    ) -> Result<stores::web::OauthToken, BoxError> {
        let token = oauth2::RefreshToken::new(token.refresh_token);

        Ok(self
            .exchange_refresh_token(&token)
            .request_async(async_http_client)
            .await?)
    }
}
