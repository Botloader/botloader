use discordoauthwrapper::DiscordOauthClient;

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response},
    middleware::Next,
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::{
    ops::Deref,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::{error, info, Instrument};

use stores::web::Session;

use crate::{app_state::AppState, errors::ApiErrorResponse};

#[derive(Clone)]
pub struct LoggedInSession {
    pub api_client: Arc<dyn DiscordOauthClient>,
    pub session: Session,
}

impl LoggedInSession {
    pub fn new(session: Session, api_client: Arc<dyn DiscordOauthClient>) -> Self {
        Self {
            api_client,
            session,
        }
    }
}

#[derive(Clone)]
pub struct OptionalSession(Option<LoggedInSession>);

impl Deref for OptionalSession {
    type Target = Option<LoggedInSession>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl OptionalSession {
    pub fn none() -> Self {
        Self(None)
    }
}

/// Looks up the session behind the Authorization header, if any, and makes it
/// available to inner services through [LoggedInSession]/[OptionalSession]
/// request extensions.
pub async fn session_mw(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<axum::response::Response, ApiErrorResponse> {
    let session = match req.headers().get("Authorization").map(|v| v.to_str()) {
        Some(Ok(token)) => state.db.get_session(token).await.map_err(|err| {
            error!(%err, "failed fetching session");
            ApiErrorResponse::InternalError
        })?,
        // treat a malformed header the same as an absent one
        Some(Err(_)) => None,
        None => None,
    };

    let Some(session) = session else {
        req.extensions_mut().insert(OptionalSession::none());
        return Ok(next.run(req).await);
    };

    let span = tracing::info_span!("session", user_id=%session.user.id);

    async {
        let api_client = state
            .oauth_api_client_cache
            .fetch(session.user.id, &session.oauth_token.access_token);

        let logged_in_session = LoggedInSession::new(session, api_client);

        // Check if the session is broken, for example they unauthorized botloader or something along those lines.
        let was_broken = check_broken_session(&state, &logged_in_session).await;
        if !was_broken {
            req.extensions_mut().insert(logged_in_session.clone());
            req.extensions_mut()
                .insert(OptionalSession(Some(logged_in_session.clone())));
        } else {
            info!("Found broken session before handler ran");
            req.extensions_mut().insert(OptionalSession(None));
        }

        let resp = next.run(req).await;

        // re-check after running the inner handler, as it could have changed
        if !was_broken {
            if check_broken_session(&state, &logged_in_session).await {
                info!("Found broken session after handler ran");
            }
        }

        Ok(resp)
    }
    .instrument(span)
    .await
}

async fn check_broken_session(
    state: &Arc<crate::app_state::InnerAppState>,
    logged_in_session: &LoggedInSession,
) -> bool {
    if !logged_in_session.api_client.is_broken() {
        return false;
    }

    // the refresh token is invalid, the user needs to re-authorize;
    // drop the client and log them out everywhere
    let user_id = logged_in_session.session.user.id;
    state.oauth_api_client_cache.del(user_id);
    state
        .db
        .del_all_sessions(user_id)
        .await
        .map_err(|err| error!(%err, "failed clearing sessions from broken token"))
        .ok();

    true
}

#[derive(Clone)]
pub struct RequireAuthLayer {}

impl<S> Layer<S> for RequireAuthLayer {
    type Service = RequireAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct RequireAuthMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequireAuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let extensions = req.extensions();
            match extensions.get::<LoggedInSession>() {
                Some(_) => inner.call(req).await.map_err(|e| e.into()),
                None => Err(NoSession(()).into()),
            }
        })
    }
}

#[derive(Debug, Default)]
pub struct NoSession(pub ());

impl fmt::Display for NoSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("no session or session expired")
    }
}

impl std::error::Error for NoSession {}
