use std::sync::Arc;

use common::{config::RunConfig, DiscordConfig};
use stores::Db;
use tracing::{info, warn};
use twilight_model::id::Id;

use crate::{
    news_poller::{self, NewsHandle},
    WebConfig,
};

pub struct InnerAppState {
    pub db: Db,
    pub web_config: WebConfig,
    pub common_config: RunConfig,
    pub stripe_client: Option<stripe_premium::Client>,
    pub discord_oauth_client: oauth2::basic::BasicClient,
    pub news_handle: NewsHandle,
    pub discord_config: Arc<DiscordConfig>,
    pub bot_rpc_client: botrpc::Client,
    pub state_client: dbrokerapi::state_client::Client,
}

pub type AppState = Arc<InnerAppState>;

pub async fn init_app_state(common_conf: &RunConfig, web_conf: &WebConfig) -> AppState {
    let postgres_store = Db::new_with_url(&common_conf.database_url).await.unwrap();

    let discord_config = common::discord::fetch_discord_config(common_conf.discord_token.clone())
        .await
        .unwrap();

    let news_handle = init_news_handle(web_conf, discord_config.clone()).await;

    let oauth_client = common_conf.get_discord_oauth2_client();

    let bot_rpc_client = botrpc::Client::new(common_conf.bot_rpc_connect_addr.clone());

    let stripe_client = init_stripe_client(postgres_store.clone(), web_conf);
    let state_client = dbrokerapi::state_client::Client::new(web_conf.broker_api_addr.clone());

    Arc::new(InnerAppState {
        web_config: web_conf.clone(),
        common_config: common_conf.clone(),
        db: postgres_store,
        stripe_client,
        discord_oauth_client: oauth_client,
        news_handle,
        discord_config,
        bot_rpc_client,
        state_client,
    })
}

async fn init_news_handle(web_conf: &WebConfig, discord_config: Arc<DiscordConfig>) -> NewsHandle {
    if let Some(guild_id) = web_conf.news_guild {
        let split = web_conf.news_channels.split(',');

        let poller = news_poller::NewsPoller::new(
            discord_config,
            split
                .into_iter()
                .map(|v| Id::new(v.parse().unwrap()))
                .collect(),
            guild_id,
        )
        .await
        .unwrap();

        let handle = poller.handle();
        info!("running news poller");
        tokio::spawn(poller.run());
        handle
    } else {
        Default::default()
    }
}

fn init_stripe_client(db: Db, config: &WebConfig) -> Option<stripe_premium::Client> {
    match (
        &config.stripe_private_key,
        &config.stripe_premium_product_id,
        &config.stripe_premium_price_id,
        &config.stripe_lite_product_id,
        &config.stripe_lite_price_id,
    ) {
        (
            Some(stripe_private_key),
            Some(stripe_premium_product_id),
            Some(stripe_premium_price_id),
            Some(stripe_lite_product_id),
            Some(stripe_lite_price_id),
        ) => {
            info!("Stripe client created");
            Some(stripe_premium::Client::new(
                db,
                stripe_private_key.to_owned(),
                stripe_lite_product_id.to_owned(),
                stripe_lite_price_id.to_owned(),
                stripe_premium_product_id.to_owned(),
                stripe_premium_price_id.to_owned(),
            ))
        }
        _ => {
            warn!("One or more stripe settings not provided");
            None
        }
    }
}
