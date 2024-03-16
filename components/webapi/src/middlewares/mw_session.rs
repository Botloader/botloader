use discordoauthwrapper::{ClientCache, DiscordOauthApiClient, TwilightApiProvider};

use axum::{
    http::{Request, Response},
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::{
    convert::Infallible,
    marker::PhantomData,
    ops::Deref,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::{error, Instrument};

use stores::web::{Session, SessionStore};

type OAuthApiClientWrapper<ST> =
    DiscordOauthApiClient<TwilightApiProvider, oauth2::basic::BasicClient, ST>;

#[derive(Clone)]
pub struct LoggedInSession<ST> {
    pub api_client: OAuthApiClientWrapper<ST>,
    pub session: Session,
}

#[derive(Clone)]
pub struct OptionalSession<ST>(Option<LoggedInSession<ST>>);

impl<ST> Deref for OptionalSession<ST> {
    type Target = Option<LoggedInSession<ST>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<ST> OptionalSession<ST> {
    pub fn none() -> Self {
        Self(None)
    }
}

impl<T> LoggedInSession<T>
where
    T: SessionStore + 'static,
{
    pub fn new(session: Session, api_client: OAuthApiClientWrapper<T>) -> Self {
        Self {
            api_client,
            session,
        }
    }
}

#[derive(Clone)]
pub struct SessionLayer<ST> {
    pub session_store: ST,
    pub oauth_conf: oauth2::basic::BasicClient,
    pub oauth_api_client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient, ST>,
}

impl<ST> SessionLayer<ST> {
    pub fn new(session_store: ST, oauth_conf: oauth2::basic::BasicClient) -> Self {
        Self {
            session_store,
            oauth_conf,
            oauth_api_client_cache: Default::default(),
        }
    }

    pub fn require_auth_layer(&self) -> RequireAuthLayer<ST> {
        RequireAuthLayer {
            _phantom: PhantomData,
        }
    }
}

impl<ST: Clone, S> Layer<S> for SessionLayer<ST> {
    type Service = SessionMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            session_store: self.session_store.clone(),
            oauth_conf: self.oauth_conf.clone(),
            oauth_api_client_cache: self.oauth_api_client_cache.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct SessionMiddleware<S, ST> {
    pub inner: S,
    pub session_store: ST,
    pub oauth_conf: oauth2::basic::BasicClient,
    pub oauth_api_client_cache: ClientCache<TwilightApiProvider, oauth2::basic::BasicClient, ST>,
}

impl<S, ST, ReqBody, ResBody> Service<Request<ReqBody>> for SessionMiddleware<S, ST>
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

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let store = self.session_store.clone();
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
pub struct RequireAuthLayer<ST> {
    _phantom: PhantomData<ST>,
}

impl<S, ST> Layer<S> for RequireAuthLayer<ST> {
    type Service = RequireAuthMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthMiddleware {
            inner,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct RequireAuthMiddleware<S, ST> {
    inner: S,
    _phantom: PhantomData<ST>,
}

impl<S, ST, ReqBody, ResBody> Service<Request<ReqBody>> for RequireAuthMiddleware<S, ST>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
    ST: Send + Sync + SessionStore + 'static,
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
            match extensions.get::<LoggedInSession<ST>>() {
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
