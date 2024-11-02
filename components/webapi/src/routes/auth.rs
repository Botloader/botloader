use std::time::Duration;

use axum::{
    extract::{self, State},
    http::{header::LOCATION, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use common::crypto::gen_token;
use entities::{
    discord_oauth_tokens,
    web_sessions::{self, SessionType},
};
use oauth2::{reqwest::async_http_client, AuthorizationCode, Scope, TokenResponse};
use sea_orm::{sea_query::OnConflict, ActiveModelTrait, ActiveValue, EntityTrait, Iterable};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use twilight_model::user::CurrentUser;

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult,
};

#[derive(Deserialize)]
pub struct ConfirmLoginQuery {
    code: String,
    state: String,
}

#[derive(Serialize)]
pub struct ConfirmLoginSuccess {
    user: CurrentUser,
    token: String,
}

#[instrument(skip_all)]
pub async fn handle_login(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let token = state.csrf_store.generate_csrf_token().await;

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
    State(state): State<AppState>,
    Json(data): Json<ConfirmLoginQuery>,
) -> ApiResult<impl IntoResponse> {
    let valid_csrf_token = state.csrf_store.check_csrf_token(&data.state).await;

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

    let session = web_sessions::ActiveModel {
        token: ActiveValue::Set(gen_token()),
        kind: ActiveValue::Set(SessionType::User),
        user_id: ActiveValue::Set(user.id.into()),
        discriminator: ActiveValue::Set(user.discriminator as i16),
        username: ActiveValue::Set(user.name.clone()),
        avatar: ActiveValue::Set(
            user.avatar
                .clone()
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        ..Default::default()
    }
    .insert(&state.seaorm_db)
    .await
    .map_err(|err| {
        error!(%err, "failed creating user session");
        ApiErrorResponse::InternalError
    })?;

    // Update token
    let new_token_entry = discord_oauth_tokens::ActiveModel {
        user_id: ActiveValue::Set(user.id.into()),
        discord_bearer_token: ActiveValue::Set(token_result.access_token().secret().clone()),
        discord_refresh_token: ActiveValue::Set(
            token_result
                .refresh_token()
                .map(|at| at.secret().clone())
                .unwrap_or_default(),
        ),
        discord_token_expires_at: ActiveValue::Set(
            (chrono::Utc::now()
                + token_result
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
        .exec(&state.seaorm_db)
        .await
        .map_err(|err| {
            error!(%err, "failed creating user session oauth token");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(ConfirmLoginSuccess {
        token: session.token,
        user,
    }))
}

#[instrument(skip(session, state))]
pub async fn handle_logout(
    State(state): State<AppState>,
    session: extract::Extension<LoggedInSession>,
) -> ApiResult<impl IntoResponse> {
    web_sessions::Entity::delete_by_id(&session.session.token)
        .exec(&state.seaorm_db)
        .await
        .map_err(|err| {
            error!(%err, "failed deleting sesison");
            ApiErrorResponse::InternalError
        })?;

    info!("Logged out a user");

    Ok(Json(()))
}
