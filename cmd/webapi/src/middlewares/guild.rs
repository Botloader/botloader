use axum::{
    extract::{FromRequest, Path, RequestParts},
    http::{Request, Response},
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::{
    fmt::Display,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Instrument;
use twilight_model::{
    guild::Permissions,
    id::{marker::GuildMarker, Id},
    user::CurrentUserGuild,
};

use stores::web::SessionStore;

use super::LoggedInSession;

#[derive(Clone)]
pub struct CurrentGuildLayer<ST> {
    pub session_store: ST,
}

impl<ST: Clone, S> Layer<S> for CurrentGuildLayer<ST> {
    type Service = CurrentGuildMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        CurrentGuildMiddleware {
            session_store: self.session_store.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct CurrentGuildMiddleware<S, ST> {
    pub inner: S,
    pub session_store: ST,
}

#[derive(Clone, serde::Deserialize, Debug)]
struct GuildPath {
    guild: u64,
}

impl<S, ST, ReqBody, ResBody> Service<Request<ReqBody>> for CurrentGuildMiddleware<S, ST>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
    ST: 'static + SessionStore + Send + Sync + Clone,
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
            let mut req_parts = RequestParts::new(req);

            let guild_path = Path::<GuildPath>::from_request(&mut req_parts).await;
            let session: Option<&LoggedInSession<ST>> =
                req_parts.extensions().map(|e| e.get()).flatten();

            let mut span = None;

            if let (Some(s), Ok(gp)) = (session, guild_path) {
                if let Some(guild_id) = Id::<GuildMarker>::new_checked(gp.guild) {
                    if let Some(g) = fetch_guild(s, guild_id).await? {
                        span = Some(tracing::debug_span!("guild", guild_id=%g.id));

                        let extensions_mut = req_parts.extensions_mut().unwrap();
                        extensions_mut.insert(g);
                    }
                }
            }

            let req = req_parts.try_into_request().unwrap();
            if let Some(s) = span {
                inner.call(req).instrument(s).await.map_err(|e| e.into())
            } else {
                inner.call(req).await.map_err(|e| e.into())
            }
        })
    }
}

async fn fetch_guild<ST: SessionStore + Send + 'static>(
    session: &LoggedInSession<ST>,
    guild_id: Id<GuildMarker>,
) -> Result<Option<CurrentUserGuild>, BoxError> {
    let user_guilds = session.api_client.current_user_guilds().await?;
    Ok(user_guilds.into_iter().find(|e| e.id == guild_id))
}

pub struct RequireCurrentGuildAuthLayer;

impl<S> Layer<S> for RequireCurrentGuildAuthLayer {
    type Service = RequireCurrentGuildAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireCurrentGuildAuthMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct RequireCurrentGuildAuthMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequireCurrentGuildAuthMiddleware<S>
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
            let current_guild = req
                .extensions()
                .get::<CurrentUserGuild>()
                .ok_or(UnknownGuildError)?;

            if !current_guild
                .permissions
                .intersects(Permissions::ADMINISTRATOR | Permissions::MANAGE_GUILD)
            {
                return Err(MissingPermsError.into());
            }

            inner.call(req).await.map_err(|e| e.into())
        })
    }
}

#[derive(Debug)]
pub struct UnknownGuildError;

impl Display for UnknownGuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown guild, not found in current_user_guilds")
    }
}

impl std::error::Error for UnknownGuildError {}

#[derive(Debug)]
pub struct MissingPermsError;

impl Display for MissingPermsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("missing permissions to manage this guild")
    }
}

impl std::error::Error for MissingPermsError {}
