use axum::{extract::Path, http::Request, middleware::Next, RequestPartsExt};

use tracing::{error, Instrument};
use twilight_model::{
    guild::Permissions,
    id::{marker::GuildMarker, Id},
    user::CurrentUserGuild,
};

use stores::web::SessionStore;

use crate::errors::ApiErrorResponse;

use super::LoggedInSession;

#[derive(Clone, serde::Deserialize, Debug)]
struct GuildPath {
    guild: u64,
}

pub async fn current_guild_injector_middleware<B, ST>(
    request: Request<B>,
    next: Next<B>,
) -> Result<axum::response::Response, ApiErrorResponse>
where
    ST: SessionStore + Send + Sync + 'static,
    B: Send,
{
    // running extractors requires a `axum::http::request::Parts`
    let (mut parts, body) = request.into_parts();

    let guild_path: Result<Path<GuildPath>, _> = parts.extract().await;
    let mut request = Request::from_parts(parts, body);

    let session: Option<&LoggedInSession<ST>> = request.extensions().get();

    let mut span = None;

    if let (Some(s), Ok(gp)) = (session, guild_path) {
        if let Some(guild_id) = Id::<GuildMarker>::new_checked(gp.guild) {
            if let Some(g) = fetch_guild(s, guild_id).await? {
                span = Some(tracing::info_span!("guild", guild_id=%g.id));
                request.extensions_mut().insert(g);
            }
        }
    }

    Ok(if let Some(s) = span {
        next.run(request).instrument(s).await
    } else {
        next.run(request).await
    })
}

async fn fetch_guild<ST: SessionStore + Send + 'static>(
    session: &LoggedInSession<ST>,
    guild_id: Id<GuildMarker>,
) -> Result<Option<CurrentUserGuild>, ApiErrorResponse> {
    let user_guilds = session
        .api_client
        .current_user_guilds()
        .await
        .map_err(|err| {
            error!(?err, "failed fetching user guild");
            ApiErrorResponse::InternalError
        })?;
    Ok(user_guilds.into_iter().find(|e| e.id == guild_id))
}

pub async fn require_current_guild_admin_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<axum::response::Response, ApiErrorResponse>
where
    B: Send,
{
    let current_guild = request
        .extensions()
        .get::<CurrentUserGuild>()
        .ok_or(ApiErrorResponse::NoActiveGuild)?;

    if !current_guild
        .permissions
        .intersects(Permissions::ADMINISTRATOR | Permissions::MANAGE_GUILD)
    {
        return Err(ApiErrorResponse::NotGuildAdmin);
    }

    Ok(next.run(request).await)
}
