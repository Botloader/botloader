use discordoauthwrapper::{DiscordOauthApiClient, TwilightApiProvider};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    Extension,
};

use entities::{discord_oauth_tokens, web_sessions, TwilightId};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use std::{convert::Infallible, ops::Deref};
use tracing::{error, Instrument};

use crate::{app_state::AppState, errors::ApiErrorResponse};

type OAuthApiClientWrapper = DiscordOauthApiClient<TwilightApiProvider, oauth2::basic::BasicClient>;

#[derive(Clone)]
pub struct LoggedInSession {
    pub api_client: OAuthApiClientWrapper,
    pub session: web_sessions::Model,
}

impl LoggedInSession {
    pub async fn load_from_db(token: &str, app_state: &AppState) -> Result<Option<Self>, DbErr> {
        let Some((session, Some(oauth_token))) = web_sessions::Entity::find_by_id(token)
            .find_also_related(discord_oauth_tokens::Entity)
            .one(&app_state.seaorm_db)
            .await?
        else {
            return Ok(None);
        };

        let api_client = app_state
            .oauth_api_client_cache
            .fetch(*session.user_id, || {
                Result::<_, Infallible>::Ok(OAuthApiClientWrapper::new_twilight(
                    *session.user_id,
                    oauth_token.discord_bearer_token.clone(),
                    app_state.discord_oauth_client.clone(),
                    app_state.seaorm_db.clone(),
                ))
            })
            .unwrap();

        Ok(Some(LoggedInSession {
            api_client,
            session,
        }))
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

pub async fn session_mw(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiErrorResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .map(|v| v.to_str().unwrap_or_default())
        .unwrap_or_default();

    if auth_header.is_empty() {
        return Ok(next.run(req).await);
    }

    let Some(logged_in_session) = LoggedInSession::load_from_db(auth_header, &state)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching session from db");
            ApiErrorResponse::InternalError
        })?
    else {
        return Err(ApiErrorResponse::SessionExpired);
    };

    let span = tracing::info_span!("session", user_id=%logged_in_session.session.user_id);

    let extensions = req.extensions_mut();

    extensions.insert(logged_in_session.clone());
    extensions.insert(OptionalSession(Some(logged_in_session.clone())));

    let resp = next.run(req).instrument(span).await;

    if logged_in_session.api_client.is_broken() {
        // remove from store if the refresh token is broken
        web_sessions::Entity::delete_many()
            .filter(web_sessions::Column::UserId.eq(TwilightId(*logged_in_session.session.user_id)))
            .exec(&state.seaorm_db)
            .await
            .map_err(|err| error!(%err, "failed clearing sessions from broken token"))
            .ok();
    }

    Ok(resp)
}

pub async fn require_auth_mw(
    session: Extension<OptionalSession>,
    request: Request,
    next: Next,
) -> Result<Response, ApiErrorResponse> {
    if session.is_none() {
        return Err(ApiErrorResponse::Unauthorized);
    }

    Ok(next.run(request).await)
}
