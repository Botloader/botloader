use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use stores::config::PremiumSlotTier;
use stripe_premium::webhook_handler::handle_stripe_webhook;
use tracing::{error, warn};

use crate::{
    app_state::AppState, errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult,
};

#[derive(Debug, Serialize)]
pub struct UrlResponse {
    url: String,
}

pub async fn handle_create_customer_portal_session(
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
) -> ApiResult<Json<UrlResponse>> {
    let Some(client) = state.stripe_client.as_ref() else {
        return Err(ApiErrorResponse::StripeNotEnabled);
    };

    let session = client
        .create_customer_portal_link(
            session.session.user.id,
            &format!("{}/user/premium", state.common_config.frontend_host_base),
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
    Extension(session): Extension<LoggedInSession>,
    State(state): State<AppState>,
    Json(body): Json<CreateCheckoutSessionBody>,
) -> ApiResult<Json<UrlResponse>> {
    let Some(client) = state.stripe_client.as_ref() else {
        return Err(ApiErrorResponse::StripeNotEnabled);
    };

    let session = client
        .create_checkout_session(
            session.session.user.id,
            body.tier,
            &format!(
                "{}/confirm_stripe_purchase",
                state.common_config.frontend_host_base
            ),
        )
        .await
        .map_err(|err| {
            tracing::error!(%err, "failed creating stripe checkout session");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(UrlResponse { url: session }))
}

pub async fn handle_webhook(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: String,
) -> Result<(), StatusCode> {
    let header = headers.get("stripe-signature").ok_or_else(|| {
        warn!("no stripe signature header on webhook endpoint");
        StatusCode::BAD_REQUEST
    })?;

    let header_str = header.to_str().map_err(|err| {
        warn!(%err, "invalid header value");
        StatusCode::BAD_REQUEST
    })?;

    let secret = state
        .web_config
        .stripe_webhook_secret
        .as_ref()
        .ok_or_else(|| {
            error!("stripe webhook endpoint hit when no secret is configured");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let client = state.stripe_client.as_ref().ok_or_else(|| {
        error!("stripe webhook endpoint hit stripe is not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    handle_stripe_webhook(client, header_str, body, secret).await?;

    Ok(())
}
