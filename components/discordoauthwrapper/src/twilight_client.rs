use std::{
    fmt::Display,
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use oauth2::reqwest::async_http_client;
use stores::{web::DiscordOauthToken, Db};
use tokio::sync::RwLock;
use twilight_http::api_error::{ApiError, RatelimitedApiError};
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::{CurrentUser, CurrentUserGuild},
};

use crate::{BoxError, DiscordOauthClient};

const RESPONSE_CACHE_DURATION: Duration = Duration::from_secs(10);

/// The real [DiscordOauthClient], calling discord through twilight and
/// refreshing tokens through the discord oauth api.
pub struct TwilightOauthClient {
    user_id: Id<UserMarker>,
    oauth_client: oauth2::basic::BasicClient,
    db: Db,

    // set when the refresh token is no longer valid
    broken: AtomicBool,

    inner: RwLock<Inner>,
}

struct Inner {
    client: twilight_http::Client,
    cached_guilds: Option<(Instant, Vec<CurrentUserGuild>)>,
    cached_user: Option<(Instant, CurrentUser)>,
}

impl TwilightOauthClient {
    pub fn new(
        user_id: Id<UserMarker>,
        access_token: String,
        oauth_client: oauth2::basic::BasicClient,
        db: Db,
    ) -> Self {
        Self {
            user_id,
            oauth_client,
            db,
            broken: AtomicBool::new(false),
            inner: RwLock::new(Inner {
                client: twilight_http::Client::new(format!("Bearer {access_token}")),
                cached_guilds: None,
                cached_user: None,
            }),
        }
    }

    async fn fetch_current_user(&self) -> Result<CurrentUser, RequestError> {
        let mut inner = self.inner.write().await;

        if let Some((t, user)) = &inner.cached_user {
            if t.elapsed() < RESPONSE_CACHE_DURATION {
                return Ok(user.clone());
            }
        }

        let resp = inner.client.current_user().await?.model().await.unwrap();

        inner.cached_user = Some((Instant::now(), resp.clone()));

        Ok(resp)
    }

    async fn fetch_current_user_guilds(&self) -> Result<Vec<CurrentUserGuild>, RequestError> {
        let mut inner = self.inner.write().await;

        if let Some((t, guilds)) = &inner.cached_guilds {
            if t.elapsed() < RESPONSE_CACHE_DURATION {
                return Ok(guilds.clone());
            }
        }

        let resp = inner
            .client
            .current_user_guilds()
            .await?
            .model()
            .await
            .unwrap();

        inner.cached_guilds = Some((Instant::now(), resp.clone()));

        Ok(resp)
    }

    // runs the provided closure, refreshing the token and retrying if needed
    async fn run_refreshing<F, FRT, Fut>(&self, f: F) -> Result<FRT, BoxError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<FRT, RequestError>>,
    {
        let mut refreshed_token = false;
        loop {
            match f().await {
                Ok(v) => return Ok(v),
                Err(RequestError::InvalidToken) => {
                    if refreshed_token {
                        self.broken.store(true, Ordering::SeqCst);

                        return Err(anyhow::anyhow!("invalid token twice").into());
                    }

                    if let Err(err) = self.refresh_token().await {
                        self.broken.store(true, Ordering::SeqCst);

                        return Err(err);
                    }
                    refreshed_token = true;
                }
                Err(RequestError::Ratelimit(dur)) => {
                    tokio::time::sleep(dur).await;
                }
                Err(RequestError::Other(e)) => return Err(e.into()),
            }
        }
    }

    async fn refresh_token(&self) -> Result<(), BoxError> {
        let current = self.db.get_oauth_token(self.user_id).await?;

        let refresh_token = oauth2::RefreshToken::new(current.refresh_token);
        let token_response = self
            .oauth_client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?;

        let new_token = DiscordOauthToken::new(self.user_id, token_response);
        let access_token = new_token.access_token.clone();
        self.db.set_user_oatuh_token(new_token).await?;

        let mut inner = self.inner.write().await;
        inner.client = twilight_http::Client::new(format!("Bearer {access_token}"));

        Ok(())
    }
}

#[async_trait::async_trait]
impl DiscordOauthClient for TwilightOauthClient {
    async fn current_user(&self) -> Result<CurrentUser, BoxError> {
        self.run_refreshing(|| self.fetch_current_user()).await
    }

    async fn current_user_guilds(&self) -> Result<Vec<CurrentUserGuild>, BoxError> {
        self.run_refreshing(|| self.fetch_current_user_guilds())
            .await
    }

    fn is_broken(&self) -> bool {
        self.broken.load(Ordering::SeqCst)
    }
}

#[derive(Debug)]
enum RequestError {
    InvalidToken,
    Ratelimit(Duration),
    Other(twilight_http::Error),
}

impl std::error::Error for RequestError {}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => f.write_str("invalid token"),
            Self::Ratelimit(dur) => f.write_fmt(format_args!("ratelimited: {dur:?}")),
            Self::Other(inner) => f.write_fmt(format_args!("{inner}")),
        }
    }
}

impl From<twilight_http::Error> for RequestError {
    fn from(te: twilight_http::Error) -> Self {
        use twilight_http::error::ErrorType as TwilightErrorType;

        match te.kind() {
            TwilightErrorType::Response { status, .. } if status.get() == 401 => Self::InvalidToken,
            TwilightErrorType::Unauthorized => Self::InvalidToken,
            TwilightErrorType::Response {
                error: ApiError::Ratelimited(RatelimitedApiError { retry_after, .. }),
                ..
            } => Self::Ratelimit(Duration::from_millis(*retry_after as u64)),
            _ => Self::Other(te),
        }
    }
}
