use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use twilight_http::api_error::{ApiError, RatelimitedApiError};
use twilight_model::user::{CurrentUser, CurrentUserGuild};

use crate::ApiProviderError;

struct Inner {
    client: twilight_http::Client,
    cached_guilds: Option<(Instant, Vec<CurrentUserGuild>)>,
    cached_user: Option<(Instant, CurrentUser)>,
}

pub struct TwilightApiProvider {
    inner: RwLock<Inner>,
}

impl TwilightApiProvider {
    pub fn new(client: twilight_http::Client) -> Self {
        Self {
            inner: RwLock::new(Inner {
                client,
                cached_guilds: None,
                cached_user: None,
            }),
        }
    }
}

#[async_trait::async_trait]
impl crate::DiscordOauthApiProvider for TwilightApiProvider {
    type OtherError = twilight_http::Error;

    async fn get_current_user(&self) -> Result<CurrentUser, ApiProviderError<Self::OtherError>> {
        let mut inner = self.inner.write().await;

        // check cache
        if let Some((t, user)) = &inner.cached_user {
            if t.elapsed() < Duration::from_secs(10) {
                return Ok(user.clone());
            }
        }

        let resp = inner
            .client
            .current_user()
            .exec()
            .await?
            .model()
            .await
            .unwrap();

        inner.cached_user = Some((Instant::now(), resp.clone()));

        Ok(resp)
    }

    async fn get_user_guilds(
        &self,
    ) -> Result<Vec<CurrentUserGuild>, ApiProviderError<Self::OtherError>> {
        let mut inner = self.inner.write().await;

        // check cache
        if let Some((t, guilds)) = &inner.cached_guilds {
            if t.elapsed() < Duration::from_secs(10) {
                return Ok(guilds.clone());
            }
        }

        let resp = inner
            .client
            .current_user_guilds()
            .exec()
            .await?
            .model()
            .await
            .unwrap();

        inner.cached_guilds = Some((Instant::now(), resp.clone()));

        Ok(resp)
    }

    async fn update_token(&self, access_token: String) {
        let new_client = twilight_http::Client::new(format!("Bearer {}", access_token));
        let mut inner = self.inner.write().await;
        inner.client = new_client;
    }
}

impl From<twilight_http::Error> for ApiProviderError<twilight_http::Error> {
    fn from(te: twilight_http::Error) -> Self {
        match te.kind() {
            twilight_http::error::ErrorType::Response {
                status,
                // The below seems to be broken
                // TODO: Debug the below
                // workaround works for now
                // error:
                //     ApiError::General(GeneralApiError {
                //         code: ErrorCode::UnknownToken | ErrorCode::InvalidOAuthAccessToken,
                //         ..
                //     }),
                ..
            } if status.get() == 401 => Self::InvalidToken,
            twilight_http::error::ErrorType::Response {
                error: ApiError::Ratelimited(RatelimitedApiError { retry_after, .. }),
                ..
            } => Self::Ratelimit(Duration::from_millis(*retry_after as u64)),
            _ => Self::Other(te),
        }
    }
}
