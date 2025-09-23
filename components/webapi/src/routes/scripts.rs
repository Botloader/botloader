use std::rc::Rc;

use axum::{
    extract::{Extension, Path, State},
    response::IntoResponse,
    Json,
};
use botrpc::{types::GuildSpecifier, BotServiceClient};
use common::plugin::Plugin;
use runtime_models::internal::script::SettingsOptionValue;
use serde::{Deserialize, Serialize};
use stores::config::{CreateScript, Script, UpdateScript};
use tracing::error;
use twilight_model::user::CurrentUserGuild;
use validation::{
    validate,
    web::{GuildData, ScriptValidationContextData},
    ValidationError,
};

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::plugins::fetch_plugin,
    util::EmptyResponse, ApiResult,
};

pub async fn get_all_guild_scripts(
    Extension(current_guild): Extension<CurrentUserGuild>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let scripts = state
        .db
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
    State(state): State<AppState>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<impl IntoResponse> {
    let scripts = state
        .db
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
        state.db.get_plugins(&fetch_plugins).await.map_err(|err| {
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
    pub script_id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRequestData {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
}

pub async fn create_guild_script(
    State(state): State<AppState>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Json(payload): Json<CreateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let cs = CreateScript {
        enabled: payload.enabled,
        original_source: payload.original_source,
        name: payload.name,
        plugin_auto_update: None,
        plugin_id: None,
        plugin_version_number: None,
    };

    if let Err(verr) = validate(&cs, &()) {
        return Err(ApiErrorResponse::ValidationFailed(verr));
    }

    if state
        .db
        .get_script(current_guild.id, cs.name.clone())
        .await
        .is_ok()
    {
        return Err(ApiErrorResponse::ValidationFailed(vec![ValidationError {
            field: "name".to_string(),
            msg: "Name already taken".to_string(),
        }]));
    }

    let script = state
        .db
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
    #[serde(default)]
    pub settings_values: Option<Vec<SettingsOptionValue>>,
}

pub async fn update_guild_script(
    Extension(current_guild): Extension<CurrentUserGuild>,
    State(state): State<AppState>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
    Json(payload): Json<UpdateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let current_script = state
        .db
        .get_script_by_id(current_guild.id, script_id)
        .await
        .map_err(|err| {
            if err.is_not_found() {
                ApiErrorResponse::ScriptNotFound
            } else {
                error!(%err, "failed fetching script");
                ApiErrorResponse::InternalError
            }
        })?;

    let sc = UpdateScript {
        id: script_id,
        enabled: payload.enabled,
        original_source: payload.original_source,
        name: payload.name,
        contributes: None,
        plugin_version_number: None,
        settings_definitions: None,
        settings_values: payload.settings_values,
    };

    {
        let validation_data = if sc.settings_definitions.is_some() {
            let channels = state
                .state_client
                .get_channels(current_guild.id)
                .await
                .map_err(|err| {
                    error!(%err, "failed fetching guild channels");
                    ApiErrorResponse::InternalError
                })?;

            let roles = state
                .state_client
                .get_roles(current_guild.id)
                .await
                .map_err(|err| {
                    error!(%err, "failed fetching guild roles");
                    ApiErrorResponse::InternalError
                })?;

            ScriptValidationContextData {
                script: current_script,
                guild_data: Some(Rc::new(GuildData { channels, roles })),
            }
        } else {
            ScriptValidationContextData {
                script: current_script,
                guild_data: None,
            }
        };

        if let Err(validation_errors) = validate(&sc, &validation_data) {
            return Err(ApiErrorResponse::ValidationFailed(validation_errors));
        }
    }

    let script = state
        .db
        .update_script(current_guild.id, sc)
        .await
        .map_err(|err| {
            error!(%err, "failed updating guild script");
            ApiErrorResponse::InternalError
        })?;

    state
        .bot_rpc_client
        .reload_vm(GuildSpecifier {
            guild_id: current_guild.id,
        })
        .await
        .map_err(|err| {
            error!(?err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}

pub async fn validate_script_settings(
    State(state): State<AppState>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
    Json(payload): Json<UpdateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let current_script = state
        .db
        .get_script_by_id(current_guild.id, script_id)
        .await
        .map_err(|err| {
            if err.is_not_found() {
                ApiErrorResponse::ScriptNotFound
            } else {
                error!(%err, "failed fetching script");
                ApiErrorResponse::InternalError
            }
        })?;

    let sc = UpdateScript {
        id: script_id,
        enabled: payload.enabled,
        original_source: payload.original_source,
        name: payload.name,
        contributes: None,
        plugin_version_number: None,
        settings_definitions: None,
        settings_values: payload.settings_values,
    };

    let validation_data = if sc.settings_definitions.is_some() {
        let channels = state
            .state_client
            .get_channels(current_guild.id)
            .await
            .map_err(|err| {
                error!(%err, "failed fetching guild channels");
                ApiErrorResponse::InternalError
            })?;

        let roles = state
            .state_client
            .get_roles(current_guild.id)
            .await
            .map_err(|err| {
                error!(%err, "failed fetching guild roles");
                ApiErrorResponse::InternalError
            })?;

        ScriptValidationContextData {
            script: current_script,
            guild_data: Some(Rc::new(GuildData { channels, roles })),
        }
    } else {
        ScriptValidationContextData {
            script: current_script,
            guild_data: None,
        }
    };

    if let Err(validation_errors) = validate(&sc, &validation_data) {
        return Err(ApiErrorResponse::ValidationFailed(validation_errors));
    }

    Ok(EmptyResponse)
}

pub async fn delete_guild_script(
    State(state): State<AppState>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
) -> ApiResult<impl IntoResponse> {
    let script = state
        .db
        .get_script_by_id(current_guild.id, script_id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild script");
            ApiErrorResponse::InternalError
        })?;

    state
        .db
        .del_script(current_guild.id, script.name.clone())
        .await
        .map_err(|err| {
            error!(%err, "failed deleting guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}

pub async fn update_script_plugin(
    State(state): State<AppState>,
    // Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
) -> ApiResult<impl IntoResponse> {
    let script = state
        .db
        .get_script_by_id(current_guild.id, script_id)
        .await
        .map_err(|err| {
            error!(?err, "failed fetching guild script");
            ApiErrorResponse::InternalError
        })?;

    let Some(plugin_id) = script.plugin_id else {
        return Err(ApiErrorResponse::ScriptNotAPlugin);
    };

    let plugin = fetch_plugin(&state.db, plugin_id).await?;
    let new_source = match plugin.data {
        common::plugin::PluginData::ScriptPlugin(p) => p.published_version.unwrap_or_default(),
    };

    // I think if we have already added a plugin to a guild then we should still be able to update it even if it's set to private afterwards
    //
    // TODO decision on this
    //
    // if !plugin.is_public && plugin.author_id != session.session.user.id {
    //     return Err(ApiErrorResponse::NoAccessToPlugin);
    // }

    let sc = UpdateScript {
        id: script_id,
        enabled: None,
        original_source: Some(new_source),
        name: None,
        contributes: None,
        plugin_version_number: Some(plugin.current_version),
        settings_definitions: None,
        settings_values: None,
    };

    let script = state
        .db
        .update_script(current_guild.id, sc)
        .await
        .map_err(|err| {
            error!(%err, "failed updating guild script");
            ApiErrorResponse::InternalError
        })?;

    state
        .bot_rpc_client
        .reload_vm(GuildSpecifier {
            guild_id: current_guild.id,
        })
        .await
        .map_err(|err| {
            error!(%err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}
