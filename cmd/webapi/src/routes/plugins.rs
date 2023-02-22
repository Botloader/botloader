use axum::{response::IntoResponse, Extension, Json};
use common::plugin::Plugin;
use serde::Deserialize;
use stores::config::{ConfigStore, ConfigStoreError, CreatePlugin, UpdatePluginMeta};
use tracing::error;
use twilight_model::user::CurrentUserGuild;
use validation::{validate, ValidationContext, Validator};

use crate::{
    errors::ApiErrorResponse,
    middlewares::{plugins::fetch_plugin, LoggedInSession},
    util::EmptyResponse,
    ApiResult, CurrentConfigStore, CurrentSessionStore,
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
pub async fn get_plugin(Extension(plugin): Extension<Plugin>) -> ApiResult<impl IntoResponse> {
    // All the logic is handled by middleware
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

// update plugin meta
pub async fn update_plugin_meta(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(plugin): Extension<Plugin>,
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

    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let plugin = config_store
        .update_plugin_meta(plugin.id, update)
        .await
        .map_err(|err| {
            error!(?err, "failed updating plugin");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(plugin))
}

#[derive(Deserialize)]
pub struct UpdatePluginDevSourceRequest {
    new_source: String,
}

impl Validator for UpdatePluginDevSourceRequest {
    fn validate(&self, ctx: &mut ValidationContext) {
        validation::web::check_script_source(ctx, "new_source", &self.new_source);
    }
}

// update plugin dev source
pub async fn update_plugin_dev_source(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(plugin): Extension<Plugin>,
    Json(body): Json<UpdatePluginDevSourceRequest>,
) -> ApiResult<impl IntoResponse> {
    if let Err(err) = validate(&body) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let plugin = config_store
        .update_script_plugin_dev_version(plugin.id, body.new_source)
        .await
        .map_err(|err| {
            error!(?err, "failed updating plugin");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(plugin))
}

// publish plugin version
#[derive(Deserialize)]
pub struct PublishPluginVersionData {
    new_source: String,
}

impl Validator for PublishPluginVersionData {
    fn validate(&self, ctx: &mut ValidationContext) {
        validation::web::check_script_source(ctx, "new_source", &self.new_source);
    }
}

pub async fn publish_plugin_version(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(plugin): Extension<Plugin>,
    Extension(bot_rpc): Extension<botrpc::Client>,
    Json(body): Json<PublishPluginVersionData>,
) -> ApiResult<impl IntoResponse> {
    if let Err(err) = validate(&body) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let guilds = config_store
        .publish_script_plugin_version(plugin.id, body.new_source)
        .await
        .map_err(|err| {
            error!(?err, "failed updating plugin");
            ApiErrorResponse::InternalError
        })?;

    // restart relevant guild vms
    // TODO: this should be done as a background task, and potentially throttled to avoid a spike
    for guild_id in guilds {
        if let Err(err) = bot_rpc.restart_guild_vm(guild_id).await {
            error!(%err, "failed reloading guild vm");
        }
    }

    Ok(EmptyResponse)
}

#[derive(Deserialize)]
pub struct GuildAddPluginData {
    plugin_id: u64,
    auto_update: bool,
}

pub async fn guild_add_plugin(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Extension(bot_rpc): Extension<botrpc::Client>,
    Json(body): Json<GuildAddPluginData>,
) -> ApiResult<impl IntoResponse> {
    let plugin = fetch_plugin(&config_store, body.plugin_id).await?;

    if !plugin.is_public && plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let script = config_store
        .try_guild_add_script_plugin(current_guild.id, plugin.id, body.auto_update)
        .await
        .map_err(|err| match err {
            ConfigStoreError::GuildAlreadyHasPlugin => ApiErrorResponse::GuildAlreadyHasPlugin,
            _ => {
                error!(?err, "failed adding plugin");
                ApiErrorResponse::InternalError
            }
        })?;

    bot_rpc
        .restart_guild_vm(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}
