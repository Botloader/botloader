use axum::extract::State;
use axum::{extract::Path, RequestPartsExt};
use axum::{extract::Request, middleware::Next, response::Response};

use common::plugin::Plugin;
use stores::Db;
use tracing::error;

use stores::config::ConfigStoreError;

use crate::app_state::AppState;
use crate::{errors::ApiErrorResponse, ApiResult};

use super::LoggedInSession;

#[derive(Clone, serde::Deserialize, Debug)]
struct PluginPath {
    plugin_id: u64,
}

pub async fn plugin_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiErrorResponse> {
    // running extractors requires a `axum::http::request::Parts`
    let (mut parts, body) = request.into_parts();

    let path: Path<PluginPath> = parts.extract().await.unwrap();
    let mut request = Request::from_parts(parts, body);

    let session: Option<&LoggedInSession> = request.extensions().get();
    let plugin = fetch_plugin(&state.db, path.plugin_id).await?;

    if !plugin.is_public {
        if let Some(session) = session {
            if plugin.author_id != *session.session.user_id {
                return Err(ApiErrorResponse::NoAccessToPlugin);
            }

            // checks passed, we have access
        } else {
            return Err(ApiErrorResponse::NoAccessToPlugin);
        }
    }

    request.extensions_mut().insert(plugin);
    Ok(next.run(request).await)
}

pub async fn fetch_plugin(config_store: &Db, plugin_id: u64) -> ApiResult<Plugin> {
    config_store.get_plugin(plugin_id).await.map_err(|err| {
        if matches!(err, ConfigStoreError::PluginNotFound(_)) {
            ApiErrorResponse::PluginNotFound
        } else {
            error!(?err, "failed fetching plugin");
            ApiErrorResponse::InternalError
        }
    })
}
