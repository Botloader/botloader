use axum::{extract::Extension, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use stores::{
    config::{ConfigStore, PremiumSlot, PremiumSlotTier},
    web::SessionStore,
};
use twilight_model::{
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    user::CurrentUserGuild,
};

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

pub async fn get_guild_settings<CT: ConfigStore + 'static>(
    Extension(config_store): Extension<CT>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<impl IntoResponse> {
    let settings = config_store
        .get_guild_meta_config_or_default(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild config");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(settings))
}

pub async fn get_guild_premium_slots<CT: ConfigStore + 'static>(
    Extension(config_store): Extension<CT>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<Json<Vec<GuildPremiumSlot>>> {
    let slots = config_store
        .get_guild_premium_slots(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild config");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(slots.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Serialize)]
pub struct GuildPremiumSlot {
    pub id: u64,
    pub title: String,
    pub user_id: Option<Id<UserMarker>>,
    pub tier: PremiumSlotTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub attached_guild_id: Option<Id<GuildMarker>>,
}

impl From<PremiumSlot> for GuildPremiumSlot {
    fn from(v: PremiumSlot) -> Self {
        Self {
            id: v.id,
            title: v.title,
            user_id: v.user_id,
            tier: v.tier,
            created_at: v.created_at,
            updated_at: v.updated_at,
            expires_at: v.expires_at,
            attached_guild_id: v.attached_guild_id,
        }
    }
}
