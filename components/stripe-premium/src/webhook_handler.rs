use std::sync::Arc;

use axum::{
    async_trait,
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use stripe::{Event, EventObject, EventType};

use crate::Client;

#[derive(Clone)]
pub struct StripeWebhookSecret(Arc<str>);

impl StripeWebhookSecret {
    pub fn new(str: String) -> Self {
        Self(Arc::from(str))
    }
}

pub struct StripeEvent(Event);

#[async_trait]
impl<S> FromRequest<S> for StripeEvent
where
    String: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let signature = if let Some(sig) = req.headers().get("stripe-signature") {
            sig.to_owned()
        } else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let secret = req
            .extensions()
            .get::<StripeWebhookSecret>()
            .unwrap()
            .0
            .clone();

        let payload = String::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        Ok(Self(
            stripe::Webhook::construct_event(&payload, signature.to_str().unwrap(), &secret)
                .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
        ))
    }
}

#[axum::debug_handler]
pub async fn handle_webhook(
    Extension(client): Extension<Arc<Option<Client>>>,
    StripeEvent(event): StripeEvent,
) -> Result<(), StatusCode> {
    let Some(client) = client.as_ref() else {
        tracing::error!("cant handle stripe webhook without a stripe client");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    match event.type_ {
        EventType::CustomerSubscriptionCreated => {
            if let EventObject::Subscription(sub) = event.data.object {
                client.sync_subscription_slots(&sub).await.map_err(|err| {
                    tracing::error!(%err, "failed handling stripe webhook CustomerSubscriptionCreated");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            }
        }
        EventType::CustomerSubscriptionDeleted => {
            if let EventObject::Subscription(sub) = event.data.object {
                client.sync_subscription_slots(&sub).await.map_err(|err| {
                    tracing::error!(%err, "failed handling stripe webhook CustomerSubscriptionDeleted");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            }
        }
        EventType::CustomerSubscriptionUpdated => {
            if let EventObject::Subscription(sub) = event.data.object {
                client.sync_subscription_slots(&sub).await.map_err(|err| {
                    tracing::error!(%err, "failed handling stripe webhook CustomerSubscriptionUpdated");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            }
        }
        _ => {}
    }

    Ok(())
}
