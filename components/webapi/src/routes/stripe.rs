use std::sync::Arc;

use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use stores::config::PremiumSlotTier;

use crate::{
    errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult, ConfigData,
    CurrentSessionStore,
};

#[derive(Debug, Serialize)]
pub struct UrlResponse {
    url: String,
}

pub async fn handle_create_customer_portal_session(
    Extension(config_data): Extension<ConfigData>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(client): Extension<Arc<Option<stripe_premium::Client>>>,
) -> ApiResult<Json<UrlResponse>> {
    let Some(client) = client.as_ref() else {
        return Err(ApiErrorResponse::StripeNotEnabled);
    };

    let session = client
        .create_customer_portal_link(
            session.session.user.id,
            &format!("{}/user/premium", config_data.frontend_base),
        )
        .await
        .map_err(|err| {
            tracing::error!(%err, "failed creating stripe customer portal session");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(UrlResponse { url: session }))
}

#[derive(Deserialize, Debug)]
pub struct CreateCheckoutSessionBody {
    tier: PremiumSlotTier,
}

pub async fn handle_create_checkout_session(
    Extension(config_data): Extension<ConfigData>,
    Extension(session): Extension<LoggedInSession<CurrentSessionStore>>,
    Extension(client): Extension<Arc<Option<stripe_premium::Client>>>,
    Json(body): Json<CreateCheckoutSessionBody>,
) -> ApiResult<Json<UrlResponse>> {
    let Some(client) = client.as_ref() else {
        return Err(ApiErrorResponse::StripeNotEnabled);
    };

    let session = client
        .create_checkout_session(
            session.session.user.id,
            body.tier,
            &format!("{}/confirm_stripe_purchase", config_data.frontend_base),
        )
        .await
        .map_err(|err| {
            tracing::error!(%err, "failed creating stripe checkout session");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(UrlResponse { url: session }))
}
