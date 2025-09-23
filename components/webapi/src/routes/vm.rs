use crate::{app_state::AppState, errors::ApiErrorResponse, util::EmptyResponse, ApiResult};
use axum::{
    extract::{Extension, State},
    response::IntoResponse,
};
use botrpc::BotServiceClient;
use tracing::error;
use twilight_model::user::CurrentUserGuild;

pub async fn reload_guild_vm(
    Extension(current_guild): Extension<CurrentUserGuild>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    state
        .bot_rpc_client
        .reload_vm(botrpc::types::GuildSpecifier {
            guild_id: current_guild.id,
        })
        .await
        .map_err(|err| {
            error!(%err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}
