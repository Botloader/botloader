use axum::{extract::Path, RequestPartsExt};
use axum::{extract::Request, middleware::Next, response::Response};

use common::plugin::Plugin;
use tracing::error;

use stores::config::{ConfigStore, ConfigStoreError};

use crate::{errors::ApiErrorResponse, ApiResult, CurrentConfigStore, CurrentSessionStore};

use super::LoggedInSession;

#[derive(Clone, serde::Deserialize, Debug)]
struct PluginPath {
    plugin_id: u64,
}

pub async fn plugin_middleware(request: Request, next: Next) -> Result<Response, ApiErrorResponse> {
    // running extractors requires a `axum::http::request::Parts`
    let (mut parts, body) = request.into_parts();

    let path: Path<PluginPath> = parts.extract().await.unwrap();
    let mut request = Request::from_parts(parts, body);

    let config_store: &CurrentConfigStore = request.extensions().get().unwrap();
    let session: Option<&LoggedInSession<CurrentSessionStore>> = request.extensions().get();

    let plugin = fetch_plugin(config_store, path.plugin_id).await?;

    if !plugin.is_public {
        if let Some(session) = session {
            if plugin.author_id != session.session.user.id {
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

pub async fn fetch_plugin(config_store: &CurrentConfigStore, plugin_id: u64) -> ApiResult<Plugin> {
    config_store.get_plugin(plugin_id).await.map_err(|err| {
        if matches!(err, ConfigStoreError::PluginNotFound(_)) {
            ApiErrorResponse::PluginNotFound
        } else {
            error!(?err, "failed fetching plugin");
            ApiErrorResponse::InternalError
        }
    })
}
