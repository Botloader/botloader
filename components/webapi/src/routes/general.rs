use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    Json,
};

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::LoggedInSession,
    news_poller::NewsItem, ApiResult,
};

use tracing::error;

/// Transparent wrapper so the schema of the foreign twilight type can be provided
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct CurrentUserResponse(
    #[schemars(with = "crate::discord_schemas::CurrentUserSchema")]
    pub  twilight_model::user::CurrentUser,
);

pub async fn get_current_user(
    Extension(session): Extension<LoggedInSession>,
) -> ApiResult<Json<CurrentUserResponse>> {
    let user = session.api_client.current_user().await.map_err(|err| {
        error!(%err, "failed fetching user");
        ApiErrorResponse::InternalError
    })?;

    Ok(Json(CurrentUserResponse(user)))
}

pub async fn get_news(State(state): State<AppState>) -> Json<Arc<Vec<NewsItem>>> {
    let latest = state.news_handle.get_items();
    Json(latest)
}
