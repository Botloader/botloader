use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use common::{plugin::Plugin, DiscordConfig};
use serde::{Deserialize, Serialize};
use stores::config::{ConfigStore, ConfigStoreError, CreatePlugin, UpdatePluginMeta};
use tracing::error;
use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::{CurrentUser, CurrentUserGuild},
};
use validation::{validate, ValidationContext, Validator};

use crate::{
    errors::ApiErrorResponse,
    middlewares::{plugins::fetch_plugin, LoggedInSession, OptionalSession},
    util::EmptyResponse,
    ApiResult, CurrentConfigStore, CurrentSessionStore,
};

#[derive(Serialize, Clone, Debug)]
pub struct DiscordUser {
    id: Id<UserMarker>,
    username: Option<String>,
    discriminator: Option<String>,
    avatar: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct User {
    #[serde(flatten)]
    inner: DiscordUser,
    is_bl_staff: bool,
    is_bl_trusted: bool,
}

#[derive(Serialize)]
pub struct PluginResponse {
    #[serde(flatten)]
    plugin: Plugin,
    author: User,
}

// get all plugins (TODO: filtering)
pub async fn get_published_public_plugins(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(discord_config): Extension<Arc<DiscordConfig>>,
    Extension(maybe_session): Extension<OptionalSession<CurrentSessionStore>>,
) -> ApiResult<Json<Vec<PluginResponse>>> {
    let plugins = config_store
        .get_published_public_plugins()
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    let plugins = fetch_plugin_authors(
        &discord_config,
        maybe_session.as_ref().map(|v| &v.session.user),
        &plugins,
    )
    .await?;

    Ok(Json(plugins))
}

// get user plugins
pub async fn get_user_plugins(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(discord_config): Extension<Arc<DiscordConfig>>,
) -> ApiResult<impl IntoResponse> {
    let plugins = config_store
        .get_user_plugins(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    let plugins =
        fetch_plugin_authors(&discord_config, Some(&session.session.user), &plugins).await?;

    Ok(Json(plugins))
}

// get plugin
pub async fn get_plugin(
    Extension(plugin): Extension<Plugin>,
    Extension(maybe_session): Extension<OptionalSession<CurrentSessionStore>>,
    Extension(discord_config): Extension<Arc<DiscordConfig>>,
) -> ApiResult<impl IntoResponse> {
    let plugin = fetch_plugin_author(
        &discord_config,
        maybe_session.as_ref().map(|v| &v.session.user),
        &plugin,
    )
    .await?;

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

pub async fn fetch_plugin_author(
    config: &DiscordConfig,
    logged_in_user: Option<&CurrentUser>,
    plugin: &Plugin,
) -> ApiResult<PluginResponse> {
    let user = fetch_discord_user(config, logged_in_user, plugin.author_id).await?;
    let is_staff = config.owners.iter().any(|v| v.id == plugin.author_id);

    Ok(PluginResponse {
        plugin: plugin.clone(),
        author: User {
            inner: user,
            is_bl_staff: is_staff,
            is_bl_trusted: is_staff,
        },
    })
}

pub async fn fetch_plugin_authors(
    config: &DiscordConfig,
    logged_in_user: Option<&CurrentUser>,
    plugins: &[Plugin],
) -> ApiResult<Vec<PluginResponse>> {
    let mut ids: Vec<_> = plugins.iter().map(|v| v.author_id).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();

    let mut fetched_users: Vec<User> = Vec::with_capacity(ids.len());
    for id in ids {
        let user = fetch_discord_user(config, logged_in_user, id).await?;
        let is_staff = config.owners.iter().any(|v| v.id == id);
        fetched_users.push(User {
            inner: user,
            is_bl_staff: is_staff,
            is_bl_trusted: is_staff,
        });
    }

    Ok(plugins
        .iter()
        .map(|v| PluginResponse {
            plugin: v.clone(),
            author: fetched_users
                .iter()
                // fetch_discord_user always returns a user on Ok, it errors out on failure
                // so all users are present at this point
                .find(|u| u.inner.id == v.author_id)
                .unwrap()
                .clone(),
        })
        .collect())
}

async fn fetch_discord_user(
    config: &DiscordConfig,
    logged_in_user: Option<&CurrentUser>,
    id: Id<UserMarker>,
) -> ApiResult<DiscordUser> {
    // shortcut if were trying to fetch the currently signed in user!
    if let Some(current_user) = &logged_in_user {
        if id == current_user.id {
            return Ok(DiscordUser {
                id,
                avatar: current_user.avatar.map(|v| v.to_string()),
                discriminator: Some(current_user.discriminator.to_string()),
                username: Some(current_user.name.clone()),
            });
        }
    }

    match config.client.user(id).await {
        Ok(v) => {
            let user = v.model().await.map_err(|err| {
                error!(?err, "failed fetching user");
                ApiErrorResponse::InternalError
            })?;
            Ok(DiscordUser {
                id,
                avatar: user.avatar.map(|v| v.to_string()),
                discriminator: Some(user.discriminator.to_string()),
                username: Some(user.name),
            })
        }
        Err(err) => match err.kind() {
            twilight_http::error::ErrorType::Response {
                error:
                    ApiError::General(GeneralApiError {
                        code: 10013, // Unknown user
                        ..
                    }),
                ..
            } => {
                // Use mock values, user was most likely deleted
                //
                // Question: should we purge deleted user's plugins from the DB?
                // is there potentially user info we might need to remove (think: gdpr?)
                Ok(DiscordUser {
                    id,
                    username: Some("Deleted user".to_owned()),
                    discriminator: Some("0000".to_owned()),
                    avatar: None,
                })
            }
            _ => {
                error!(?err, "failed fetching user");
                Err(ApiErrorResponse::InternalError)
            }
        },
    }
}
