use axum::{
    extract::{Extension, State},
    response::IntoResponse,
};
use tracing::error;
use twilight_model::user::CurrentUserGuild;

use crate::{app_state::AppState, errors::ApiErrorResponse, util::EmptyResponse, ApiResult};

pub async fn reload_guild_vm(
    Extension(current_guild): Extension<CurrentUserGuild>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    state
        .bot_rpc_client
        .restart_guild_vm(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}
