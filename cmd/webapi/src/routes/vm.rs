use axum::{extract::Extension, response::IntoResponse};
use stores::web::SessionStore;
use tracing::error;
use twilight_model::user::CurrentUserGuild;

use crate::{errors::ApiErrorResponse, util::EmptyResponse, ApiResult};

pub async fn reload_guild_vm<ST: SessionStore + Clone + Send + Sync + 'static>(
    Extension(bot_rpc): Extension<botrpc::Client>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<impl IntoResponse> {
    bot_rpc
        .restart_guild_vm(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;

    Ok(EmptyResponse)
}
