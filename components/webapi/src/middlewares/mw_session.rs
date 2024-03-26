use discordoauthwrapper::{ClientCache, DiscordOauthApiClient, TwilightApiProvider};

use axum::{
    http::{Request, Response},
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::{
    convert::Infallible,
    ops::Deref,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::{error, Instrument};

use stores::{web::Session, Db};

type OAuthApiClientWrapper = DiscordOauthApiClient<TwilightApiProvider, oauth2::basic::BasicClient>;

#[derive(Clone)]
pub struct LoggedInSession {
    pub api_client: OAuthApiClientWrapper,
    pub session: Session,
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

impl LoggedInSession {
    pub fn new(session: Session, api_client: OAuthApiClientWrapper) -> Self {
        Self {
            api_client,
            session,
        }
    }
}

#[derive(Clone)]
pub struct SessionLayer {
    pub session_store: Db,
    pub oauth_conf: oauth2::basic::BasicClient,
    pub oauth_api_client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient>,
}

impl SessionLayer {
    pub fn new(session_store: Db, oauth_conf: oauth2::basic::BasicClient) -> Self {
        Self {
            session_store,
            oauth_conf,
            oauth_api_client_cache: Default::default(),
        }
    }

    pub fn require_auth_layer(&self) -> RequireAuthLayer {
        RequireAuthLayer {}
    }
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            db: self.session_store.clone(),
            oauth_conf: self.oauth_conf.clone(),
            oauth_api_client_cache: self.oauth_api_client_cache.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct SessionMiddleware<S> {
    pub inner: S,
    pub db: Db,
    pub oauth_conf: oauth2::basic::BasicClient,
    pub oauth_api_client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SessionMiddleware<S>
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

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let store = self.db.clone();
        let oauth_conf = self.oauth_conf.clone();
        let cache = self.oauth_api_client_cache.clone();

        Box::pin(async move {
            let auth_header = req.headers().get("Authorization");

            match auth_header.map(|e| e.to_str()) {
                Some(Ok(t)) => {
                    if let Some(session) = store.get_session(t).await? {
                        let extensions = req.extensions_mut();

                        let span = tracing::info_span!("session", user_id=%session.user.id);

                        let api_client = cache
                            .fetch(session.user.id, || {
                                Result::<_, Infallible>::Ok(OAuthApiClientWrapper::new_twilight(
                                    session.user.id,
                                    session.oauth_token.access_token.clone(),
                                    oauth_conf,
                                    store.clone(),
                                ))
                            })
                            .unwrap();

                        let logged_in_session = LoggedInSession::new(session, api_client);
                        extensions.insert(logged_in_session.clone());
                        extensions.insert(OptionalSession(Some(logged_in_session.clone())));

                        let resp = {
                            // catch potential work being made creating the future
                            let _guard = span.enter();
                            let fut = inner.call(req);
                            drop(_guard);

                            fut
                        }
                        .instrument(span)
                        .await
                        .map_err(|e| e.into());

                        if logged_in_session.api_client.is_broken() {
                            // remove from store if the refresh token is broken
                            store
                                .del_all_sessions(logged_in_session.session.user.id)
                                .await
                                .map_err(|err| {
                                    error!(%err, "failed clearing sessions from broken token")
                                }).ok();
                        }

                        resp
                    } else {
                        inner.call(req).await.map_err(|e| e.into())
                    }
                }
                Some(Err(e)) => Err(e.into()),
                None => inner.call(req).await.map_err(|e| e.into()),
            }

            // if let Some(s) = span {
            //     inner.call(req).instrument(s).await.map_err(|e| e.into())
            // } else {
            //     inner.call(req).await.map_err(|e| e.into())
            // }
        })
    }
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
