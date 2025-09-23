use axum::{
    extract::{Path, State},
    Json,
};
use botrpc::BotServiceClient;
use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::GuildMarker, Id};

use crate::{app_state::AppState, errors::ApiErrorResponse, ApiResult};

use tracing::error;

pub async fn get_worker_statuses(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ApiVmWorkerStatus>>> {
    let response = state
        .bot_rpc_client
        .vm_worker_status()
        .await
        .map_err(|err| {
            error!(%err, "failed retrieving vm worker statuses");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(
        response
            .workers
            .into_iter()
            .map(|v| ApiVmWorkerStatus {
                worker_id: v.worker_id,
                currently_claimed_by_guild_id: v
                    .currently_claimed_by_guild_id
                    .map(|v| v.to_string()),
                last_claimed_by_guild_id: v.last_claimed_by_guild_id.map(|v| v.to_string()),
                claimed_last_ms_ago: v.claimed_last_ms_ago,
                returned_last_ms_ago: v.returned_last_ms_ago,
            })
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize)]
pub struct GuildIdParam {
    pub guild_id: u64,
}

pub async fn get_guild_status(
    Path(params): Path<GuildIdParam>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiGuildStatusResponse>> {
    let Some(guild_id) = Id::new_checked(params.guild_id) else {
        return Err(ApiErrorResponse::NoActiveGuild);
    };

    let status = state
        .bot_rpc_client
        .guild_status(botrpc::types::GuildSpecifier { guild_id })
        .await
        .map_err(|err| {
            error!(%err, "failed retrieving guild status");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(ApiGuildStatusResponse {
        guild_id,
        current_claimed_worker_id: status.current_claimed_worker_id,
        last_claimed_worker_id: status.last_claimed_worker_id,
        claimed_last_since_ms: status.claimed_last_since_ms,
        returned_last_since_ms: status.returned_last_since_ms,
        pending_acks: status.pending_acks,
    }))
}

#[derive(Serialize)]
pub struct ApiVmWorkerStatus {
    pub worker_id: u32,
    pub currently_claimed_by_guild_id: Option<String>,
    pub last_claimed_by_guild_id: Option<String>,
    pub claimed_last_ms_ago: u64,
    pub returned_last_ms_ago: u64,
}

#[derive(Serialize)]
pub struct ApiGuildStatusResponse {
    pub guild_id: Id<GuildMarker>,
    pub current_claimed_worker_id: Option<u32>,
    pub last_claimed_worker_id: Option<u32>,
    pub claimed_last_since_ms: u64,
    pub returned_last_since_ms: u64,
    pub pending_acks: u32,
}
