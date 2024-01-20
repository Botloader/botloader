use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use dbrokerapi::state_client::ConnectedGuildsResponse;
use tracing::info;
use twilight_cache_inmemory::{model::CachedGuild, InMemoryCache};
use twilight_model::{channel::Channel, guild::Role, id::Id};

#[derive(Clone)]
struct ReadyTracker {
    ready: Arc<AtomicBool>,
}

pub async fn run_http_server(
    conf: crate::BrokerConfig,
    discord_state: Arc<InMemoryCache>,
    ready: Arc<AtomicBool>,
) {
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
        .route("/connected_guilds", get(handle_get_connected_guilds))
        .layer(Extension(discord_state))
        .layer(Extension(ReadyTracker { ready }))
        .layer(axum_metrics_layer::MetricsLayer {
            name: "bl.broker.http_api_hits_total",
        });

    // let make_service = app.into_make_service();

    // run it with hyper on configured address
    info!("Starting hype on address: {}", conf.http_api_listen_addr);

    let listener = tokio::net::TcpListener::bind(conf.http_api_listen_addr)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_get_guild(
    Path(guild_id_u): Path<u64>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<CachedGuild>>), String> {
    let guild_id = Id::new_checked(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;

    if let Some(g) = discord_state.guild(guild_id) {
        return Ok((StatusCode::OK, Json(Some(g.value().clone()))));
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_channel(
    Path((guild_id_u, channel_id_u)): Path<(u64, u64)>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<Channel>>), String> {
    let guild_id = Id::new_checked(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;
    let channel_id = Id::new_checked(channel_id_u).ok_or_else(|| String::from("bad channel_id"))?;

    if let Some(c) = discord_state.channel(channel_id) {
        if let Some(channel_guild_id) = c.guild_id {
            if channel_guild_id == guild_id {
                return Ok((StatusCode::OK, Json(Some(c.value().clone()))));
            }
        }
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_channels(
    Path(guild_id_u): Path<u64>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<Vec<Channel>>>), String> {
    let guild_id = Id::new_checked(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;

    if let Some(c) = discord_state.guild_channels(guild_id) {
        let conv = c
            .value()
            .iter()
            .filter_map(|v| discord_state.channel(*v).map(|c| c.value().clone()));

        return Ok((StatusCode::OK, Json(Some(conv.collect()))));
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_role(
    Path((guild_id_u, role_id_u)): Path<(u64, u64)>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Result<(StatusCode, Json<Option<Role>>), String> {
    let guild_id = Id::new_checked(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;
    let role_id = Id::new_checked(role_id_u).ok_or_else(|| String::from("bad role_id"))?;

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
    let guild_id = Id::new_checked(guild_id_u).ok_or_else(|| String::from("bad guild_id"))?;

    if let Some(c) = discord_state.guild_roles(guild_id) {
        let conv = c
            .value()
            .iter()
            .filter_map(|v| discord_state.role(*v).map(|c| c.value().resource().clone()));

        return Ok((StatusCode::OK, Json(Some(conv.collect()))));
    }

    Ok((StatusCode::NOT_FOUND, Json(None)))
}

async fn handle_get_connected_guilds(
    Extension(ready_tracker): Extension<ReadyTracker>,
    Extension(discord_state): Extension<Arc<InMemoryCache>>,
) -> Json<ConnectedGuildsResponse> {
    if !ready_tracker.ready.load(Ordering::SeqCst) {
        return Json(ConnectedGuildsResponse::NotReady);
    }

    let guilds = discord_state
        .iter()
        .guilds()
        .map(|v| v.id())
        .collect::<Vec<_>>();

    Json(ConnectedGuildsResponse::Ready(guilds))
}
