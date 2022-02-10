use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::get,
    AddExtensionLayer, Json, Router,
};
use tracing::info;
use twilight_cache_inmemory::{model::CachedGuild, InMemoryCache};
use twilight_model::{
    channel::GuildChannel,
    guild::Role,
    id::{ChannelId, GuildId, RoleId},
};

pub async fn run_http_server(conf: crate::BrokerConfig, discord_state: Arc<InMemoryCache>) {
    let app = Router::new()
        // .route("/guilds/:guild_id/stream_events", get(handle_stream_events))
        .route("/guilds/:guild_id/roles", get(handle_get_roles))
        .route("/guilds/:guild_id/roles/:role_id", get(handle_get_role))
        .route("/guilds/:guild_id/channels", get(handle_get_channels))
        .route(
            "/guilds/:guild_id/channels/:channel_id",
            get(handle_get_channel),
        )
        .route("/guilds/:guild_id", get(handle_get_guild))
        .layer(AddExtensionLayer::new(discord_state))
        .layer(axum_metrics_layer::MetricsLayer {
            name: "bl.broker.http_api_hits_total",
        });

    let make_service = app.into_make_service();

    // run it with hyper on configured address
    info!("Starting hype on address: {}", conf.http_api_listen_addr);
    let addr = conf.http_api_listen_addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(common::shutdown::wait_shutdown_signal())
        .await
        .unwrap();
}

async fn handle_get_guild(
    Path(guild_id_u): Path<u64>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<CachedGuild>>), String> {
    let guild_id = GuildId::new(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;

    if let Some(g) = discord_state.guild(guild_id) {
        return Ok((StatusCode::OK, Json(Some(g.value().clone()))));
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_channel(
    Path((guild_id_u, channel_id_u)): Path<(u64, u64)>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<GuildChannel>>), String> {
    let guild_id = GuildId::new(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;
    let channel_id = ChannelId::new(channel_id_u).ok_or_else(|| String::from("bad channel_id"))?;

    if let Some(c) = discord_state.guild_channel(channel_id) {
        if c.guild_id() == guild_id {
            return Ok((StatusCode::OK, Json(Some(c.value().resource().clone()))));
        }
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_channels(
    Path(guild_id_u): Path<u64>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<Vec<GuildChannel>>>), String> {
    let guild_id = GuildId::new(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;

    if let Some(c) = discord_state.guild_channels(guild_id) {
        let conv = c.value().iter().filter_map(|v| {
            discord_state
                .guild_channel(*v)
                .map(|c| c.value().resource().clone())
        });

        return Ok((StatusCode::OK, Json(Some(conv.collect()))));
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_role(
    Path((guild_id_u, role_id_u)): Path<(u64, u64)>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<Role>>), String> {
    let guild_id = GuildId::new(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;
    let role_id = RoleId::new(role_id_u).ok_or_else(|| String::from("bad role_id"))?;

    if let Some(c) = discord_state.role(role_id) {
        if c.guild_id() == guild_id {
            return Ok((StatusCode::OK, Json(Some(c.value().resource().clone()))));
        }
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_roles(
    Path(guild_id_u): Path<u64>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<Vec<Role>>>), String> {
    let guild_id = GuildId::new(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;

    if let Some(c) = discord_state.guild_roles(guild_id) {
        let conv = c
            .value()
            .iter()
            .filter_map(|v| discord_state.role(*v).map(|c| c.value().resource().clone()));

        return Ok((StatusCode::OK, Json(Some(conv.collect()))));
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}
