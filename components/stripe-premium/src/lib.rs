use std::str::FromStr;

use chrono::DateTime;
use stores::{
    config::{
        ConfigStoreError, CreateUpdatePremiumSlotBySource, PremiumSlotState, PremiumSlotTier,
    },
    Db,
};
use stripe::{
    BillingPortalSession, CheckoutSession, CheckoutSessionMode, CreateBillingPortalSession,
    CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsAdjustableQuantity, CreateCustomer, Customer, CustomerId,
    ListSubscriptions, Subscription, SubscriptionStatusFilter,
};
use tracing::instrument;
use twilight_model::id::{marker::UserMarker, Id};

pub mod webhook_handler;

#[derive(Clone)]
pub struct Client {
    client: stripe::Client,
    db: Db,

    lite_product_id: String,
    lite_price_id: String,
    premium_product_id: String,
    premium_price_id: String,
}

impl Client {
    pub fn new(
        db: Db,
        secret_key: String,
        lite_product_id: String,
        lite_price_id: String,
        premium_product_id: String,
        premium_price_id: String,
    ) -> Self {
        let client = stripe::Client::new(secret_key);
        Self {
            client,
            db,
            lite_product_id,
            premium_product_id,
            lite_price_id,
            premium_price_id,
        }
    }

    pub async fn create_checkout_session(
        &self,
        discord_user_id: Id<UserMarker>,
        tier: PremiumSlotTier,
        return_url: &str,
    ) -> anyhow::Result<String> {
        let customer_id = self.init_user_stripe_customer_id(discord_user_id).await?;

        let price = match tier {
            PremiumSlotTier::Lite => &self.lite_price_id,
            PremiumSlotTier::Premium => &self.premium_price_id,
        };

        let session = CheckoutSession::create(
            &self.client,
            CreateCheckoutSession {
                customer: Some(CustomerId::from_str(&customer_id).unwrap()),
                mode: Some(CheckoutSessionMode::Subscription),
                success_url: Some(return_url),
                cancel_url: Some(return_url),
                line_items: Some(vec![CreateCheckoutSessionLineItems {
                    adjustable_quantity: Some(CreateCheckoutSessionLineItemsAdjustableQuantity {
                        enabled: true,
                        minimum: Some(1),
                        ..Default::default()
                    }),
                    price: Some(price.to_owned()),
                    quantity: Some(1),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        )
        .await?;

        Ok(session.url.unwrap())
    }

    #[instrument(skip_all)]
    pub async fn create_customer_portal_link(
        &self,
        discord_user_id: Id<UserMarker>,
        return_url: &str,
    ) -> anyhow::Result<String> {
        let customer_id = self.init_user_stripe_customer_id(discord_user_id).await?;
        let session = BillingPortalSession::create(
            &self.client,
            CreateBillingPortalSession {
                customer: CustomerId::from_str(&customer_id).unwrap(),
                configuration: None,
                expand: &[],
                flow_data: None,
                locale: None,
                on_behalf_of: None,
                return_url: Some(return_url),
            },
        )
        .await?;

        Ok(session.url)
    }

    async fn init_user_stripe_customer_id(
        &self,
        discord_user_id: Id<UserMarker>,
    ) -> anyhow::Result<String> {
        let user = self.db.get_user_meta(discord_user_id.get()).await?;
        if let Some(user) = user {
            if let Some(stripe_customer_id) = user.stripe_customer_id {
                return Ok(stripe_customer_id.to_string());
            }
        }

        let description = format!("Discord Id: {}", discord_user_id);
        let customer = Customer::create(
            &self.client,
            CreateCustomer {
                description: Some(&description),
                ..Default::default()
            },
        )
        .await?;

        if let Some(new_user_meta) = self
            .db
            .try_set_stripe_customer_id(discord_user_id, &customer.id)
            .await?
        {
            // Was successfully updated
            Ok(new_user_meta.stripe_customer_id.unwrap())
        } else {
            // Uh oh! It was already set!
            Customer::delete(&self.client, &customer.id).await?;

            let user = self.db.get_user_meta(discord_user_id.get()).await?;
            if let Some(user) = user {
                if let Some(stripe_customer_id) = user.stripe_customer_id {
                    return Ok(stripe_customer_id.to_string());
                }
            }

            unreachable!()
        }
    }

    #[instrument(skip_all)]
    pub async fn scan_update_discord_user_stripe_subscriptions(
        &self,
        discord_user_id: Id<UserMarker>,
    ) -> anyhow::Result<()> {
        let customer_id = self.init_user_stripe_customer_id(discord_user_id).await?;
        self.scan_update_all_stripe_subscriptions(Some(
            CustomerId::from_str(&customer_id).unwrap(),
        ))
        .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn scan_update_all_stripe_subscriptions(
        &self,
        customer_id: Option<CustomerId>,
    ) -> anyhow::Result<()> {
        let subs = Subscription::list(
            &self.client,
            &ListSubscriptions {
                limit: Some(100),
                status: Some(SubscriptionStatusFilter::All),
                customer: customer_id,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        for sub in &subs.data {
            self.sync_subscription_slots(sub).await?;
        }

        Ok(())
    }

    pub async fn sync_subscription_slots(&self, sub: &Subscription) -> anyhow::Result<()> {
        let slots = self.get_create_update_premium_slots(sub).await?;
        tracing::info!("updating {} lite/premium slots", slots.len());

        for slot in slots {
            self.db.create_update_premium_slot_by_source(slot).await?;
        }

        Ok(())
    }

    pub async fn get_create_update_premium_slots(
        &self,
        sub: &Subscription,
    ) -> Result<Vec<CreateUpdatePremiumSlotBySource>, ConfigStoreError> {
        let customer_id = sub.customer.id();
        let Some(user) = self
            .db
            .get_user_by_stripe_customer_id(customer_id.as_str())
            .await?
        else {
            tracing::warn!(
                stripe_customer_id = %customer_id,
                "unknown stripe customer for subscription"
            );
            return Ok(Vec::new());
        };

        let mut result = Vec::new();
        for item in &sub.items.data {
            let product_id = item.price.as_ref().unwrap().product.as_ref().unwrap().id();
            let quantity = item.quantity.unwrap_or(1);

            let tier = if product_id.as_str() == self.premium_product_id {
                Some(PremiumSlotTier::Premium)
            } else if product_id.as_str() == self.lite_product_id {
                Some(PremiumSlotTier::Lite)
            } else {
                None
            };

            let Some(tier) = tier else {
                tracing::info!(item_id = %item.id, "skipping unknown subscription item product");
                continue;
            };

            for i in 0..quantity {
                result.push(CreateUpdatePremiumSlotBySource {
                    title: "Stripe subscription".to_owned(),
                    user_id: twilight_model::id::Id::new_checked(user.discord_user_id),
                    message: format!(
                        "{} subscription through stripe",
                        match tier {
                            PremiumSlotTier::Lite => "Lite",
                            PremiumSlotTier::Premium => "Premium",
                        }
                    ),
                    source: "stripe_subscription".to_owned(),
                    source_id: format!("stripe_{}_{}_{}", sub.id, item.id, i),
                    tier,
                    state: match sub.status {
                        stripe::SubscriptionStatus::Active => PremiumSlotState::Active,
                        stripe::SubscriptionStatus::Canceled => PremiumSlotState::Cancelled,
                        stripe::SubscriptionStatus::Incomplete => PremiumSlotState::PaymentFailed,
                        stripe::SubscriptionStatus::IncompleteExpired => {
                            PremiumSlotState::Cancelled
                        }
                        stripe::SubscriptionStatus::PastDue => PremiumSlotState::PaymentFailed,
                        stripe::SubscriptionStatus::Paused => PremiumSlotState::Cancelled,
                        stripe::SubscriptionStatus::Trialing => PremiumSlotState::Active,
                        stripe::SubscriptionStatus::Unpaid => PremiumSlotState::Cancelled,
                    },
                    expires_at: DateTime::from_timestamp(sub.current_period_end, 0).unwrap(),
                    manage_url: String::new(),
                })
            }
        }

        Ok(result)
    }
}
