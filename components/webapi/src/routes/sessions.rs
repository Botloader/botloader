use axum::{
    extract::{Extension, State},
    response::IntoResponse,
    Json,
};
use common::crypto::gen_token;
use entities::web_sessions::{self, SessionType};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::LoggedInSession,
    util::EmptyResponse, ApiResult,
};

use tracing::error;

#[derive(Serialize)]
pub struct SessionMeta {
    kind: SessionType,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<web_sessions::Model> for SessionMeta {
    fn from(s: web_sessions::Model) -> Self {
        Self {
            created_at: s.created_at.to_utc(),
            kind: s.kind,
        }
    }
}

#[derive(Serialize)]
pub struct SessionMetaWithKey {
    kind: SessionType,
    created_at: chrono::DateTime<chrono::Utc>,
    token: String,
}

impl From<web_sessions::Model> for SessionMetaWithKey {
    fn from(s: web_sessions::Model) -> Self {
        Self {
            created_at: s.created_at.to_utc(),
            kind: s.kind,
            token: s.token,
        }
    }
}

pub async fn get_all_sessions(
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<SessionMeta>>> {
    let sessions = web_sessions::Entity::find()
        .filter(web_sessions::Column::UserId.eq(session.session.user_id))
        .all(&state.seaorm_db)
        .await
        .map_err(|err| {
            error!(%err, "failed retrieving all sessions");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(sessions.into_iter().map(|e| e.into()).collect()))
}

pub async fn create_api_token(
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
) -> ApiResult<Json<SessionMetaWithKey>> {
    let new_session = web_sessions::ActiveModel {
        token: ActiveValue::Set(gen_token()),
        kind: ActiveValue::Set(SessionType::ApiKey),
        user_id: ActiveValue::Set(session.session.user_id),
        discriminator: ActiveValue::Set(session.session.discriminator),
        username: ActiveValue::Set(session.session.username),
        avatar: ActiveValue::Set(session.session.avatar),
        ..Default::default()
    };

    let new_session = new_session.insert(&state.seaorm_db).await.map_err(|err| {
        error!(%err, "failed creating all api key");
        ApiErrorResponse::InternalError
    })?;

    Ok(Json(new_session.into()))
}

#[derive(Deserialize)]
pub struct DelSessionPayload {
    token: String,
}

pub async fn del_session(
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
    Json(payload): Json<DelSessionPayload>,
) -> ApiResult<impl IntoResponse> {
    web_sessions::Entity::delete_by_id(&payload.token)
        .filter(web_sessions::Column::UserId.eq(session.session.user_id))
        .exec(&state.seaorm_db)
        .await
        .map_err(|err| {
            error!(%err, "failed deleting session");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}

pub async fn del_all_sessions(
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    web_sessions::Entity::delete_many()
        .filter(web_sessions::Column::UserId.eq(session.session.user_id))
        .exec(&state.seaorm_db)
        .await
        .map_err(|err| {
            error!(%err, "failed deleting all sessions");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}
