use std::sync::Arc;

use axum::{
    extract::{self, State},
    http::{header::LOCATION, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use oauth2::{reqwest::async_http_client, AuthorizationCode, Scope, TokenResponse};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use twilight_model::user::CurrentUser;

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult,
};

use stores::{inmemory::web::InMemoryCsrfStore, web::DiscordOauthToken, Db};

pub struct AuthHandlers {
    db: Db,
    csrf_store: InMemoryCsrfStore,
}

#[derive(Deserialize)]
pub struct ConfirmLoginQuery {
    code: String,
    state: String,
}
impl AuthHandlers {
    pub fn new(db: Db, csrf_store: InMemoryCsrfStore) -> Self {
        Self { csrf_store, db }
    }
}

#[derive(Serialize)]
pub struct ConfirmLoginSuccess {
    user: CurrentUser,
    token: String,
}

impl AuthHandlers {
    #[instrument(skip_all)]
    pub async fn handle_login(
        auth_handler: extract::Extension<Arc<AuthHandlers>>,
        State(state): State<AppState>,
    ) -> ApiResult<impl IntoResponse> {
        let token = auth_handler.csrf_store.generate_csrf_token().await;

        // Generate the full authorization URL.
        let (auth_url, _) = state
            .discord_oauth_client
            .authorize_url(|| token)
            // Set the desired scopes.
            .add_scope(Scope::new("identify".to_string()))
            .add_scope(Scope::new("guilds".to_string()))
            // Set the PKCE code challenge.
            // .set_pkce_challenge(pkce_challenge)
            // TODO: Do we need to use pkce challenges? wouldn't it be enough to verify the "state" parameter alone?
            .url();

        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, HeaderValue::from_str(auth_url.as_ref()).unwrap());
        Ok((StatusCode::SEE_OTHER, headers))
    }

    #[instrument(skip_all)]
    pub async fn handle_confirm_login(
        auth_handler: extract::Extension<Arc<AuthHandlers>>,
        State(state): State<AppState>,
        Json(data): Json<ConfirmLoginQuery>,
    ) -> ApiResult<impl IntoResponse> {
        let valid_csrf_token = auth_handler.csrf_store.check_csrf_token(&data.state).await;

        if !valid_csrf_token {
            return Err(ApiErrorResponse::BadCsrfToken);
        }

        let token_result = state
            .discord_oauth_client
            .exchange_code(AuthorizationCode::new(data.code))
            // Set the PKCE code verifier.
            .request_async(async_http_client)
            .await
            .map_err(|err| {
                error!(%err, "failed exchanging oauth2 code");
                ApiErrorResponse::InternalError
            })?;

        let access_token = token_result.access_token();
        let client = twilight_http::Client::new(format!("Bearer {}", access_token.secret()));
        let user = client
            .current_user()
            .await
            .map_err(|err| {
                error!(%err, "discord api request failed, failed getting current user");
                ApiErrorResponse::InternalError
            })?
            .model()
            .await
            .map_err(|err| {
                error!(%err, "failed reading/decoding discord response body");
                ApiErrorResponse::InternalError
            })?;

        let session = auth_handler
            .db
            .set_oauth_create_session(
                DiscordOauthToken::new(user.id, token_result),
                user.clone(),
                stores::web::SessionType::User,
            )
            .await
            .map_err(|err| {
                error!(%err, "failed creating user session");
                ApiErrorResponse::InternalError
            })?;

        Ok(Json(ConfirmLoginSuccess {
            token: session.token,
            user,
        }))
    }

    #[instrument(skip(auth_handler, session))]
    pub async fn handle_logout(
        auth_handler: extract::Extension<Arc<AuthHandlers>>,
        session: extract::Extension<LoggedInSession>,
    ) -> ApiResult<impl IntoResponse> {
        auth_handler
            .db
            .del_session(&session.session.token)
            .await
            .map_err(|err| {
                error!(%err, "failed deleting sesison");
                ApiErrorResponse::InternalError
            })?;

        info!("Logged out a user");

        Ok(Json(()))
    }
}
