use axum::extract::State;
use axum::Extension;
use axum::{extract::Request, middleware::Next, response::Response};

use tracing::error;

use crate::app_state::AppState;
use crate::errors::ApiErrorResponse;

use super::LoggedInSession;

pub async fn bl_admin_only_mw(
    State(state): State<AppState>,
    session: Extension<LoggedInSession>,
    request: Request,
    next: Next,
) -> Result<Response, ApiErrorResponse> {
    let user_meta = state
        .db
        .get_user_meta(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(%err, "failed fetching user_meta");
            ApiErrorResponse::InternalError
        })?;

    if !user_meta.map(|v| v.is_admin).unwrap_or(false) {
        return Err(ApiErrorResponse::NotBlAdmin);
    }

    Ok(next.run(request).await)
}
