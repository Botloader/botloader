use axum::Extension;
use axum::{extract::Request, middleware::Next, response::Response};

use tracing::error;

use stores::config::ConfigStore;

use crate::{errors::ApiErrorResponse, CurrentConfigStore, CurrentSessionStore};

use super::LoggedInSession;

pub async fn bl_admin_only_mw(
    session: Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(config_store): Extension<CurrentConfigStore>,
    request: Request,
    next: Next,
) -> Result<Response, ApiErrorResponse> {
    let user_meta = config_store
        .get_user_meta(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(%err, "failed fetching user_meta");
            ApiErrorResponse::InternalError
        })?;

    if !user_meta.is_admin {
        return Err(ApiErrorResponse::NotBlAdmin);
    }

    Ok(next.run(request).await)
}
