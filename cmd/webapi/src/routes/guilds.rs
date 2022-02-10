use axum::{extract::Extension, response::IntoResponse, Json};
use stores::{config::ConfigStore, web::SessionStore};
use twilight_model::user::CurrentUserGuild;

use crate::{errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult};

use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
pub struct GuildList {
    guilds: Vec<GuildListEntry>,
}

#[derive(Serialize)]
pub struct GuildListEntry {
    connected: bool,
    guild: CurrentUserGuild,
}

pub async fn list_user_guilds_route<ST: SessionStore + 'static, CT: ConfigStore + 'static>(
    Extension(config_store): Extension<CT>,
    Extension(session): Extension<LoggedInSession<ST>>,
) -> ApiResult<impl IntoResponse> {
    let user_guilds = session
        .api_client
        .current_user_guilds()
        .await
        .map_err(|err| {
            error!(%err, "failed fetching user guilds");
            ApiErrorResponse::InternalError
        })?;

    let guild_ids = user_guilds.iter().map(|g| g.id).collect::<Vec<_>>();

    let connected_guilds = config_store
        .get_joined_guilds(&guild_ids)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching connected guilds");
            ApiErrorResponse::InternalError
        })?;

    let result = user_guilds
        .into_iter()
        .map(|g| {
            let connected = connected_guilds.iter().any(|e| e.id == g.id);
            GuildListEntry {
                connected,
                guild: g,
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(GuildList { guilds: result }))
}
