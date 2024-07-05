use std::io::Cursor;

use axum::{
    extract::{Multipart, Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use common::{
    plugin::{Plugin, PluginImageKind},
    DiscordConfig,
};
use image::{codecs::webp::WebPEncoder, GenericImageView, ImageError, Limits};
use serde::{Deserialize, Serialize};
use stores::config::{
    ConfigStoreError, CreateImage, CreatePlugin, CreateUpdatePluginImage, UpdatePluginMeta,
};
use tracing::error;
use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::{CurrentUser, CurrentUserGuild},
};
use uuid::Uuid;
use validation::{validate, ValidationContext, Validator};

use crate::{
    app_state::AppState,
    errors::ApiErrorResponse,
    middlewares::{plugins::fetch_plugin, LoggedInSession, OptionalSession},
    util::EmptyResponse,
    ApiResult,
};

#[derive(Serialize, Clone, Debug)]
pub struct DiscordUser {
    id: Id<UserMarker>,
    username: String,
    discriminator: String,
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
    State(state): State<AppState>,
    Extension(maybe_session): Extension<OptionalSession>,
) -> ApiResult<Json<Vec<PluginResponse>>> {
    let plugins = state
        .db
        .get_published_public_plugins()
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    let plugins = fetch_plugin_authors(
        &state.discord_config,
        maybe_session.as_ref().map(|v| &v.session.user),
        &plugins,
    )
    .await?;

    Ok(Json(plugins))
}

// get user plugins
pub async fn get_user_plugins(
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
) -> ApiResult<impl IntoResponse> {
    let plugins = state
        .db
        .get_user_plugins(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    let plugins =
        fetch_plugin_authors(&state.discord_config, Some(&session.session.user), &plugins).await?;

    Ok(Json(plugins))
}

// get plugin
pub async fn get_plugin(
    Extension(plugin): Extension<Plugin>,
    Extension(maybe_session): Extension<OptionalSession>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let plugin = fetch_plugin_author(
        &state.discord_config,
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
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
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

    if let Err(err) = validate(&create, &()) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    let plugins = state
        .db
        .get_user_plugins(session.session.user.id.get())
        .await
        .map_err(|err| {
            error!(?err, "failed fetching plugins");
            ApiErrorResponse::InternalError
        })?;

    if plugins.len() > 50 {
        return Err(ApiErrorResponse::UserPluginLimitReached);
    }

    let plugin = state.db.create_plugin(create).await.map_err(|err| {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_published: Option<bool>,
}

// update plugin meta
pub async fn update_plugin_meta(
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
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
        is_published: body.is_published,
        discord_thread_id: None,
    };

    if let Err(err) = validate(&update, &()) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let plugin = state
        .db
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
    type ContextData = ();
    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        validation::web::check_script_source(ctx, "new_source", &self.new_source);
    }
}

// update plugin dev source
pub async fn update_plugin_dev_source(
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
    Extension(plugin): Extension<Plugin>,
    Json(body): Json<UpdatePluginDevSourceRequest>,
) -> ApiResult<impl IntoResponse> {
    if let Err(err) = validate(&body, &()) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let plugin = state
        .db
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
    type ContextData = ();

    fn validate(&self, ctx: &mut ValidationContext, _: &()) {
        validation::web::check_script_source(ctx, "new_source", &self.new_source);
    }
}

pub async fn publish_plugin_version(
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
    Extension(plugin): Extension<Plugin>,
    Json(body): Json<PublishPluginVersionData>,
) -> ApiResult<impl IntoResponse> {
    if let Err(err) = validate(&body, &()) {
        return Err(ApiErrorResponse::ValidationFailed(err));
    }

    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let guilds = state
        .db
        .publish_script_plugin_version(plugin.id, body.new_source)
        .await
        .map_err(|err| {
            error!(?err, "failed updating plugin");
            ApiErrorResponse::InternalError
        })?;

    // restart relevant guild vms
    // TODO: this should be done as a background task, and potentially throttled to avoid a spike
    for guild_id in guilds {
        if let Err(err) = state.bot_rpc_client.restart_guild_vm(guild_id).await {
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
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Json(body): Json<GuildAddPluginData>,
) -> ApiResult<impl IntoResponse> {
    let plugin = fetch_plugin(&state.db, body.plugin_id).await?;

    if !plugin.is_public && plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    let script = state
        .db
        .try_guild_add_script_plugin(current_guild.id, plugin.id, body.auto_update)
        .await
        .map_err(|err| match err {
            ConfigStoreError::GuildAlreadyHasPlugin => ApiErrorResponse::GuildAlreadyHasPlugin,
            _ => {
                error!(?err, "failed adding plugin");
                ApiErrorResponse::InternalError
            }
        })?;

    state
        .bot_rpc_client
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
                discriminator: current_user.discriminator.to_string(),
                username: current_user.name.clone(),
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
                discriminator: user.discriminator.to_string(),
                username: user.name,
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
                    username: "Deleted user".to_owned(),
                    discriminator: "0000".to_owned(),
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

#[derive(Deserialize)]
pub struct AddPluginImageFormData {
    description: Option<String>,
    kind: PluginImageKind,
}

pub async fn add_plugin_image(
    Extension(plugin): Extension<Plugin>,
    State(state): State<AppState>,
    Extension(session): Extension<LoggedInSession>,
    mut multipart: Multipart,
) -> ApiResult<EmptyResponse> {
    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    // let mut kind: Option<PluginImageKind> = None;
    // let mut description: String = String::new();
    let mut data: Option<Vec<u8>> = None;
    let mut form: Option<AddPluginImageFormData> = None;

    while let Some(field) = multipart.next_field().await? {
        let Some(name) = field.name() else {
            continue;
        };

        if name == "form" {
            let text = field.text().await?;
            let deserialized: AddPluginImageFormData =
                serde_json::from_str(&text).map_err(ApiErrorResponse::MultipartFormJson)?;
            form = Some(deserialized);
        } else {
            let field_data = field.bytes().await?;
            data = Some(field_data.to_vec());
        }
    }

    let Some(data) = data else {
        return Err(ApiErrorResponse::MissingMultiPartFormField(
            "image".to_string(),
        ));
    };

    let Some(form) = form else {
        return Err(ApiErrorResponse::MissingMultiPartFormField(
            "form".to_string(),
        ));
    };

    match form.kind {
        PluginImageKind::Icon | PluginImageKind::Banner => {}
        PluginImageKind::Showcase => {
            let current_count = plugin
                .images
                .iter()
                .filter(|v| matches!(v.kind, PluginImageKind::Showcase))
                .count();

            if current_count >= 5 {
                return Err(ApiErrorResponse::MaxImagesReached);
            }
        }
    }

    // Transcode image to webp
    // TODO: skip if the image is already webp
    let buf = Cursor::new(&data);
    let mut reader = image::ImageReader::new(buf);
    let mut limits = Limits::default();
    limits.max_image_width = Some(1920);
    limits.max_image_height = Some(1080);
    reader.limits(limits);
    reader = reader.with_guessed_format().map_err(|err| {
        tracing::warn!(%err, "failed guessing image format");
        ApiErrorResponse::ImageNotSupported
    })?;

    let decoded = reader.decode().map_err(|err| match err {
        ImageError::Limits(_) => ApiErrorResponse::ImageTooBig,
        _ => {
            tracing::warn!(%err, "failed decoding image");
            ApiErrorResponse::ImageNotSupported
        }
    })?;

    let (width, height) = decoded.dimensions();

    let mut dst = Vec::new();
    let encoder = WebPEncoder::new_lossless(&mut dst);
    decoded.write_with_encoder(encoder).map_err(|err| {
        tracing::error!(%err, "failed encoding image");
        ApiErrorResponse::InternalError
    })?;

    let image_id = state
        .db
        .create_image(CreateImage {
            bytes: dst,
            width,
            height,
            plugin_id: plugin.id,
            user_id: session.session.user.id.get(),
        })
        .await
        .map_err(|err| {
            tracing::error!(%err, "failed saving image to db");
            ApiErrorResponse::InternalError
        })?;

    state
        .db
        .upsert_plugin_image(
            plugin.id,
            CreateUpdatePluginImage {
                image_id,
                description: form.description.unwrap_or_default(),
                kind: form.kind,
                position: 0,
            },
        )
        .await
        .map_err(|err| {
            tracing::error!(%err, "failed setting plugin image");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}

#[derive(Deserialize)]
pub struct ImageParam {
    pub image_id: Uuid,
}

pub async fn delete_plugin_image(
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
    Extension(plugin): Extension<Plugin>,
    Path(ImageParam { image_id }): Path<ImageParam>,
) -> ApiResult<EmptyResponse> {
    if plugin.author_id != session.session.user.id {
        return Err(ApiErrorResponse::NoAccessToPlugin);
    }

    state
        .db
        .delete_plugin_image(plugin.id, image_id)
        .await
        .map_err(|err| {
            tracing::error!(%err, "failed deleting plugin image");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}

#[derive(Deserialize)]
pub struct PluginImagesParam {
    pub plugin_id: u64,
    pub image_id_specifier_with_extension: String,
}

// To be honest im not sure what the deal with this lint is,
// from googling it it seems to have generated a lot of false positives in the past
// considering its just a lint and not a compiler error its also safe to ignore.
#[allow(clippy::declare_interior_mutable_const)]
const WEBP_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("image/webp");

pub async fn get_plugin_image(
    State(state): State<AppState>,
    Path(PluginImagesParam {
        plugin_id,
        image_id_specifier_with_extension,
    }): Path<PluginImagesParam>,
) -> ApiResult<(StatusCode, HeaderMap, Vec<u8>)> {
    dbg!(&image_id_specifier_with_extension);

    let Some(image_id_raw) = image_id_specifier_with_extension.strip_suffix(".webp") else {
        return Err(ApiErrorResponse::ImageNotFound);
    };

    let Ok(image_id) = Uuid::parse_str(image_id_raw) else {
        return Err(ApiErrorResponse::ImageNotFound);
    };

    let image = state
        .db
        .get_plugin_image(plugin_id, image_id)
        .await
        .map_err(|err| match err {
            ConfigStoreError::ImageNotFound(_, _) => ApiErrorResponse::ImageNotSupported,
            other => {
                tracing::error!(%other, "failed retrieving plugin image");
                ApiErrorResponse::InternalError
            }
        })?;

    let Some(image_bytes) = image.bytes else {
        return Err(ApiErrorResponse::ImageNotFound);
    };

    if image.deleted_at.is_some() {
        return Err(ApiErrorResponse::ImageNotFound);
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, WEBP_CONTENT_TYPE);

    Ok((StatusCode::OK, headers, image_bytes))
}
