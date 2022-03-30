use axum::{
    extract::{Extension, Path},
    Json,
};
use stores::{
    config::{ConfigStore, PremiumSlot},
    web::SessionStore,
};
use twilight_model::id::{marker::GuildMarker, Id};

use crate::{errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult};

use serde::Deserialize;
use tracing::error;

pub async fn list_user_premium_slots<ST: SessionStore + 'static, CT: ConfigStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
    Extension(config_store): Extension<CT>,
) -> ApiResult<Json<Vec<PremiumSlot>>> {
    let slots = config_store
        .get_user_premium_slots(session.session.user.id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching user premium slots");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(slots))
}

pub async fn update_premium_slot_guild<ST: SessionStore + 'static, CT: ConfigStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
    Extension(config_store): Extension<CT>,
    Extension(bot_rpc): Extension<botrpc::Client>,
    Path(UpdateSlotPathParams { slot_id }): Path<UpdateSlotPathParams>,
    Json(body): Json<UpdateSlotGuildBody>,
) -> ApiResult<Json<PremiumSlot>> {
    let res = config_store
        .update_premium_slot_attachment(session.session.user.id, slot_id, body.guild_id)
        .await
        .map_err(|err| {
            error!(%err, "failed updating premium slot");
            ApiErrorResponse::InternalError
        })?;

    if let Some(guild_id) = res.attached_guild_id {
        bot_rpc.restart_guild_vm(guild_id).await.map_err(|err| {
            error!(%err, "failed reloading guild vm");
            ApiErrorResponse::InternalError
        })?;
    }

    Ok(Json(res))
}

#[derive(Deserialize)]
pub struct UpdateSlotGuildBody {
    guild_id: Option<Id<GuildMarker>>,
}

#[derive(Deserialize)]
pub struct UpdateSlotPathParams {
    slot_id: u64,
}
