use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    Json,
};
use common::plugin::Plugin;
use serde::{Deserialize, Serialize};
use stores::config::{ConfigStore, CreateScript, Script, UpdateScript};
use tracing::error;
use twilight_model::user::CurrentUserGuild;
use validation::validate;

use crate::{errors::ApiErrorResponse, ApiResult, CurrentConfigStore};

pub async fn get_all_guild_scripts(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<impl IntoResponse> {
    let scripts = config_store
        .list_scripts(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild scripts");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(scripts))
}

#[derive(Serialize)]
pub struct GetScriptsWithPluginsResponse {
    scripts: Vec<Script>,
    plugins: Vec<Plugin>,
}

pub async fn get_all_guild_scripts_with_plugins(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<impl IntoResponse> {
    let scripts = config_store
        .list_scripts(current_guild.id)
        .await
        .map_err(|err| {
            error!(?err, "failed fetching guild scripts");
            ApiErrorResponse::InternalError
        })?;

    let fetch_plugins = scripts
        .iter()
        .filter_map(|v| v.plugin_id)
        .collect::<Vec<_>>();

    let plugins = if !fetch_plugins.is_empty() {
        config_store
            .get_plugins(&fetch_plugins)
            .await
            .map_err(|err| {
                error!(?err, "failed fetching plugins");
                ApiErrorResponse::InternalError
            })?
    } else {
        Vec::new()
    };

    Ok(Json(GetScriptsWithPluginsResponse { plugins, scripts }))
}

#[derive(Deserialize)]
pub struct GuildScriptPathParams {
    script_id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRequestData {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
}

pub async fn create_guild_script(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Json(payload): Json<CreateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let cs = CreateScript {
        enabled: payload.enabled,
        original_source: payload.original_source,
        name: payload.name,
        plugin_auto_update: None,
        plugin_id: None,
    };

    if let Err(verr) = validate(&cs) {
        return Err(ApiErrorResponse::ValidationFailed(verr));
    }

    let script = config_store
        .create_script(current_guild.id, cs)
        .await
        .map_err(|err| {
            error!(%err, "failed creating guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRequestData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub original_source: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

pub async fn update_guild_script(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
    Json(payload): Json<UpdateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let sc = UpdateScript {
        id: script_id,
        enabled: payload.enabled,
        original_source: payload.original_source,
        name: payload.name,
        contributes: None,
    };

    if let Err(verr) = validate(&sc) {
        return Err(ApiErrorResponse::ValidationFailed(verr));
    }

    let script = config_store
        .update_script(current_guild.id, sc)
        .await
        .map_err(|err| {
            error!(%err, "failed updating guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}

pub async fn delete_guild_script(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
) -> ApiResult<impl IntoResponse> {
    let script = config_store
        .get_script_by_id(current_guild.id, script_id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild script");
            ApiErrorResponse::InternalError
        })?;

    config_store
        .del_script(current_guild.id, script.name.clone())
        .await
        .map_err(|err| {
            error!(%err, "failed deleting guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}
