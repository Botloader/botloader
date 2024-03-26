use axum::{
    extract::{Extension, State},
    response::IntoResponse,
    Json,
};

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult,
};

use tracing::error;

pub async fn get_current_user(
    Extension(session): Extension<LoggedInSession>,
) -> ApiResult<impl IntoResponse> {
    let user = session.api_client.current_user().await.map_err(|err| {
        error!(%err, "failed fetching user");
        ApiErrorResponse::InternalError
    })?;

    Ok(Json(user))
}

pub async fn get_news(State(state): State<AppState>) -> impl IntoResponse {
    let latest = state.news_handle.get_items();
    Json(latest)
}
