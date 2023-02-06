use axum::{extract::Path, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use stores::config::{ConfigStore, ConfigStoreError, CreatePlugin, UpdatePluginMeta};
use tracing::error;
use validation::validate;

use crate::{
    errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult, CurrentConfigStore,
    CurrentSessionStore,
};

// get all plugins (TODO: filtering)
pub async fn get_published_public_plugins(
    Extension(config_store): Extension<CurrentConfigStore>,
) -> ApiResult<impl IntoResponse> {
    let plugins = config_store
        .get_published_public_plugins()
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(plugins))
}

// get user plugins
pub async fn get_user_plugins(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
) -> ApiResult<impl IntoResponse> {
    let plugins = config_store
        .get_user_plugins(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(plugins))
}

// get plugin
pub async fn get_plugin(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Path(plugin_id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let plugin = config_store.get_plugin(plugin_id).await.map_err(|err| {
        if matches!(err, ConfigStoreError::PluginNotFound(_)) {
            ApiErrorResponse::PluginNotFound
        } else {
            error!(?err, "failed fetching plugin");
            ApiErrorResponse::InternalError
        }
    })?;

    if !plugin.is_public && plugin.author_id != session.session.user.id.get() {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    Ok(Json(plugin))
}

// create plugin
#[derive(Deserialize)]
pub struct CreatePluginBody {
    pub name: String,
    pub short_description: String,
    pub long_description: String,
}

pub async fn create_plugin(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Json(body): Json<CreatePluginBody>,
) -> ApiResult<impl IntoResponse> {
    let create = CreatePlugin {
        author_id: session.session.user.id.get(),
        name: body.name,
        short_description: body.short_description,
        long_description: body.long_description,
        is_official: false,
        is_public: false,
        kind: common::plugin::PluginType::Script,
    };

    if let Err(err) = validate(&create) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    let plugins = config_store
        .get_user_plugins(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    if plugins.len() > 50 {
        return Err(ApiErrorResponse::UserPluginLimitReached);
    }

    let plugin = config_store.create_plugin(create).await.map_err(|err| {
        error!(?err, "failed creating plugin");
        ApiErrorResponse::InternalError
    })?;

    Ok(Json(plugin))
}

#[derive(Deserialize)]
pub struct UpdatePluginMetaRequest {
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub is_public: Option<bool>,
}

// update plugin meta
pub async fn update_plugin_meta(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Path(plugin_id): Path<u64>,
    Json(body): Json<UpdatePluginMetaRequest>,
) -> ApiResult<impl IntoResponse> {
    let update = UpdatePluginMeta {
        name: body.name,
        short_description: body.short_description,
        long_description: body.long_description,
        is_public: body.is_public,
        is_official: None,
        author_id: None,
        is_published: None,
    };

    if let Err(err) = validate(&update) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    let plugin = config_store.get_plugin(plugin_id).await.map_err(|err| {
        if matches!(err, ConfigStoreError::PluginNotFound(_)) {
            ApiErrorResponse::PluginNotFound
        } else {
            error!(?err, "failed fetching plugin");
            ApiErrorResponse::InternalError
        }
    })?;

    if plugin.author_id != session.session.user.id.get() {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let plugin = config_store
        .update_plugin_meta(plugin_id, update)
        .await
        .map_err(|err| {
            error!(?err, "failed updating plugin");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(plugin))
}

// update plugin dev source
// publish plugin version
