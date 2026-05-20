use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_extra::extract::Query;
use dbrokerapi::{
    models::{BrokerEmoji, BrokerGuild},
    state_client::ConnectedGuildsResponse,
};
use serde::Deserialize;
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;
use twilight_cache_inmemory::{model::CachedGuild, InMemoryCache};
use twilight_model::{
    channel::Channel,
    guild::{Member, Role},
    id::Id,
    voice::VoiceState,
};

use crate::broker::{BrokerHandle, GuildMembersRequest};

enum ApiError {
    BadGuildId,
    BadChannelId,
    BadRoleId,
    MemberChunkTimeout,
    GuildNotFound,
    ChannelNotFound,
    RoleNotFound,
    BrokerShuttingDown,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::BadGuildId => (StatusCode::BAD_REQUEST, "Bad guild id").into_response(),
            ApiError::BadChannelId => (StatusCode::BAD_REQUEST, "Bad channel id").into_response(),
            ApiError::BadRoleId => (StatusCode::BAD_REQUEST, "Bad role id").into_response(),
            ApiError::MemberChunkTimeout => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Member chunk timed out").into_response()
            }
            ApiError::GuildNotFound => (StatusCode::NOT_FOUND, "Guild not found").into_response(),
            ApiError::ChannelNotFound => {
                (StatusCode::NOT_FOUND, "Channel not found").into_response()
            }
            ApiError::RoleNotFound => (StatusCode::NOT_FOUND, "Role not found").into_response(),
            ApiError::BrokerShuttingDown => {
                (StatusCode::BAD_REQUEST, "Broker is shutting down").into_response()
            }
        }
    }
}

type ApiResult<T> = Result<T, ApiError>;

struct InnerRouterState {
    discord_state: Arc<InMemoryCache>,
    ready_tracker: Arc<AtomicBool>,
    broker_handle: BrokerHandle,
}

type RouterState = Arc<InnerRouterState>;

pub async fn run_http_server(
    conf: crate::BrokerConfig,
    discord_state: Arc<InMemoryCache>,
    ready: Arc<AtomicBool>,
    broker_handle: BrokerHandle,
) {
    let app = Router::new()
        // .route("/guilds/:guild_id/stream_events", get(handle_stream_events))
        .route("/guilds/:guild_id", get(handle_get_guild))
        .route(
            "/guilds/:guild_id/voice_states",
            get(handle_get_guild_voice_states),
        )
        .route("/guilds/:guild_id/emojis", get(handle_get_emojis))
        .route("/guilds/:guild_id/channels", get(handle_get_channels))
        .route(
            "/guilds/:guild_id/channels/:channel_id",
            get(handle_get_channel),
        )
        .route("/guilds/:guild_id/members", get(handle_get_members))
        .route("/guilds/:guild_id/roles", get(handle_get_roles))
        .route("/guilds/:guild_id/roles/:role_id", get(handle_get_role))
        .route("/connected_guilds", get(handle_get_connected_guilds))
        .with_state(RouterState::new(InnerRouterState {
            broker_handle,
            discord_state,
            ready_tracker: ready,
        }))
        .layer(axum_metrics_layer::MetricsLayer {
            name_prefix: "bl.broker",
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
    State(state): State<RouterState>,
) -> ApiResult<Json<BrokerGuild>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;

    if let Some(g) = state.discord_state.guild(guild_id) {
        let broker_guild = BrokerGuild {
            id: g.id(),
            name: g.name().to_string(),
            icon: g.icon().map(|v| v.to_string()),
            splash: g.splash().map(|v| v.to_string()),
            discovery_splash: g.discovery_splash().map(|v| v.to_string()),
            owner_id: g.owner_id(),
            afk_channel_id: g.afk_channel_id(),
            afk_timeout: g.afk_timeout().get() as u64,
            verification_level: g.verification_level(),
            explicit_content_filter: g.explicit_content_filter(),
            mfa_level: g.mfa_level(),
            application_id: g.application_id(),
            system_channel_id: g.system_channel_id(),
            widget_enabled: g.widget_enabled(),
            widget_channel_id: g.widget_channel_id(),
            rules_channel_id: g.rules_channel_id(),
            preferred_locale: g.preferred_locale().to_string(),
            premium_tier: g.premium_tier(),
            premium_subscription_count: g.premium_subscription_count(),
            banner: g.banner().map(|v| v.to_string()),
            default_message_notifications: g.default_message_notifications(),
            description: g.description().map(|v| v.to_string()),
            features: g
                .features()
                .map(|v| Cow::from(v.clone()).to_string())
                .collect(),
            joined_at: g.joined_at(),
            large: g.large(),
            max_members: g.max_members(),
            max_presences: g.max_presences(),
            member_count: g.member_count(),
            nsfw_level: g.nsfw_level(),
            owner: g.owner(),
            permissions: g.permissions(),
            premium_progress_bar_enabled: g.premium_progress_bar_enabled(),
            system_channel_flags: g.system_channel_flags(),
            unavailable: g.unavailable().unwrap_or(false),
            vanity_url_code: g.vanity_url_code().map(|v| v.to_owned()),
        };

        return Ok(Json(broker_guild));
    }

    Err(ApiError::GuildNotFound)
}

async fn handle_get_guild_voice_states(
    Path(guild_id_u): Path<u64>,
    State(state): State<RouterState>,
) -> ApiResult<Json<Vec<VoiceState>>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;

    let Some(users) = state.discord_state.guild_voice_states(guild_id) else {
        return Err(ApiError::GuildNotFound);
    };

    let mut result: Vec<VoiceState> = Vec::with_capacity(users.len());
    for user in users.iter() {
        let Some(voice_state) = state.discord_state.voice_state(*user, guild_id) else {
            continue;
        };

        result.push(VoiceState {
            channel_id: Some(voice_state.channel_id()),
            guild_id: Some(guild_id),
            deaf: voice_state.deaf(),
            member: None,
            mute: voice_state.mute(),
            self_deaf: voice_state.self_deaf(),
            self_mute: voice_state.self_mute(),
            self_stream: voice_state.self_stream(),
            self_video: voice_state.self_video(),
            session_id: voice_state.session_id().to_owned(),
            suppress: voice_state.suppress(),
            user_id: voice_state.user_id(),
            request_to_speak_timestamp: voice_state.request_to_speak_timestamp(),
        });
    }

    Ok(Json(result))
}

async fn handle_get_channel(
    Path((guild_id_u, channel_id_u)): Path<(u64, u64)>,
    State(state): State<RouterState>,
) -> ApiResult<Json<Channel>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;
    let channel_id = Id::new_checked(channel_id_u).ok_or(ApiError::BadChannelId)?;

    if let Some(c) = state.discord_state.channel(channel_id) {
        if let Some(channel_guild_id) = c.guild_id {
            if channel_guild_id == guild_id {
                return Ok(Json(c.value().clone()));
            }
        }
    }

    Err(ApiError::ChannelNotFound)
}

async fn handle_get_channels(
    Path(guild_id_u): Path<u64>,
    State(state): State<RouterState>,
) -> ApiResult<Json<Vec<Channel>>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;

    if let Some(c) = state.discord_state.guild_channels(guild_id) {
        let conv = c
            .value()
            .iter()
            .filter_map(|v| state.discord_state.channel(*v).map(|c| c.value().clone()));

        return Ok(Json(conv.collect()));
    }

    Err(ApiError::GuildNotFound)
}

async fn handle_get_emojis(
    Path(guild_id_u): Path<u64>,
    State(state): State<RouterState>,
) -> ApiResult<Json<Vec<BrokerEmoji>>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;

    if let Some(c) = state.discord_state.guild_emojis(guild_id) {
        let conv = c.value().iter().filter_map(|v| {
            state.discord_state.emoji(*v).map(|c| BrokerEmoji {
                id: c.id(),
                animated: c.animated(),
                available: c.available(),
                managed: c.managed(),
                name: c.name().to_string(),
                require_colons: c.require_colons(),
                roles: c.roles().to_vec(),
                user_id: c.user_id(),
            })
        });

        return Ok(Json(conv.collect()));
    }

    Err(ApiError::GuildNotFound)
}

async fn handle_get_role(
    Path((guild_id_u, role_id_u)): Path<(u64, u64)>,
    State(state): State<RouterState>,
) -> ApiResult<Json<Role>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;
    let role_id = Id::new_checked(role_id_u).ok_or(ApiError::BadRoleId)?;

    if let Some(c) = state.discord_state.role(role_id) {
        if c.guild_id() == guild_id {
            return Ok(Json(c.value().resource().clone()));
        }
    }

    Err(ApiError::RoleNotFound)
}

async fn handle_get_roles(
    Path(guild_id_u): Path<u64>,
    State(state): State<RouterState>,
) -> ApiResult<Json<Vec<Role>>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;

    if let Some(c) = state.discord_state.guild_roles(guild_id) {
        let conv = c.value().iter().filter_map(|v| {
            state
                .discord_state
                .role(*v)
                .map(|c| c.value().resource().clone())
        });

        return Ok(Json(conv.collect()));
    }

    Err(ApiError::GuildNotFound)
}

async fn handle_get_connected_guilds(
    State(state): State<RouterState>,
) -> Json<ConnectedGuildsResponse> {
    if !state.ready_tracker.load(Ordering::SeqCst) {
        return Json(ConnectedGuildsResponse::NotReady);
    }

    let guilds = state
        .discord_state
        .iter()
        .guilds()
        .map(|v| v.id())
        .collect::<Vec<_>>();

    Json(ConnectedGuildsResponse::Ready(guilds))
}

#[derive(Debug, Deserialize)]
struct GetGuildMembersQuery {
    #[serde(rename = "user_id")]
    user_ids: Vec<u64>,
}

async fn handle_get_members(
    Path(guild_id_u): Path<u64>,
    State(state): State<RouterState>,
    Query(query): Query<GetGuildMembersQuery>,
) -> ApiResult<Json<Vec<Member>>> {
    let guild_id = Id::new_checked(guild_id_u).ok_or(ApiError::BadGuildId)?;

    let user_ids = query
        .user_ids
        .iter()
        .filter_map(|v| Id::new_checked(*v))
        .collect::<Vec<_>>();

    let (tx, mut rx) = unbounded_channel();
    if let Err(err) = state
        .broker_handle
        .send(crate::broker::BrokerCommand::RequestGuildMembers(
            GuildMembersRequest {
                guild_id,
                user_ids,
                response: tx,
            },
        ))
    {
        tracing::error!(
            ?err,
            "failed fetching gateway members, broker is not running"
        );

        return Err(ApiError::BrokerShuttingDown);
    }

    let mut received_members = Vec::new();
    loop {
        match tokio::time::timeout(tokio::time::Duration::from_secs(10), rx.recv()).await {
            Ok(Some(mut chunk)) => {
                received_members.append(&mut chunk);
            }
            Ok(None) => break,
            Err(_) => return Err(ApiError::MemberChunkTimeout),
        }
    }

    while let Some(mut chunk) = rx.recv().await {
        received_members.append(&mut chunk);
    }

    Ok(Json(received_members))
}
