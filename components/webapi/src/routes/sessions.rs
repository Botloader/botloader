use axum::{extract::Extension, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use stores::web::{Session, SessionStore, SessionType};

use crate::{
    errors::ApiErrorResponse, middlewares::LoggedInSession, util::EmptyResponse, ApiResult,
};

use tracing::error;

#[derive(Serialize)]
pub struct SessionMeta {
    kind: SessionType,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Session> for SessionMeta {
    fn from(s: Session) -> Self {
        Self {
            created_at: s.created_at,
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

impl From<Session> for SessionMetaWithKey {
    fn from(s: Session) -> Self {
        Self {
            created_at: s.created_at,
            kind: s.kind,
            token: s.token,
        }
    }
}

pub async fn get_all_sessions<ST: SessionStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
    Extension(session_store): Extension<ST>,
) -> ApiResult<Json<Vec<SessionMeta>>> {
    let sessions = session_store
        .get_all_sessions(session.session.user.id)
        .await
        .map_err(|err| {
            error!(%err, "failed retrieving all sessions");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(sessions.into_iter().map(|e| e.into()).collect()))
}

pub async fn create_api_token<ST: SessionStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
    Extension(session_store): Extension<ST>,
) -> ApiResult<Json<SessionMetaWithKey>> {
    let session = session_store
        .create_session(session.session.user.clone(), SessionType::ApiKey)
        .await
        .map_err(|err| {
            error!(%err, "failed creating all api key");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(session.into()))
}

#[derive(Deserialize)]
pub struct DelSessionPayload {
    token: String,
}

pub async fn del_session<ST: SessionStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
    Extension(session_store): Extension<ST>,
    Json(payload): Json<DelSessionPayload>,
) -> ApiResult<impl IntoResponse> {
    let deleting = session_store
        .get_session(&payload.token)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching session");
            ApiErrorResponse::InternalError
        })?;

    // TODO: return proper error
    match deleting {
        Some(s) => {
            if s.user.id != session.session.user.id {
                return Err(ApiErrorResponse::InternalError);
            }
        }
        None => {
            return Err(ApiErrorResponse::InternalError);
        }
    }

    session_store
        .del_session(&payload.token)
        .await
        .map_err(|err| {
            error!(%err, "failed deleting session");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}

pub async fn del_all_sessions<ST: SessionStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
    Extension(session_store): Extension<ST>,
) -> ApiResult<impl IntoResponse> {
    session_store
        .del_all_sessions(session.session.user.id)
        .await
        .map_err(|err| {
            error!(%err, "failed deleting all sessions");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}
