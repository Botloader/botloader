use std::sync::Arc;

use structopt::StructOpt;
use tracing::info;
use twilight_cache_inmemory::InMemoryCacheBuilder;

use crate::{broker::run_broker, http_api::run_http_server};

mod broker;
mod dispatch_server;
mod http_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::common_init(Some("0.0.0.0:7802"));
    let config = BrokerConfig::from_args();
    // let discord_config = common::fetch_discord_config(config.common.discord_token.clone())
    //     .await
    //     .expect("failed fetching discord config");

    info!("Launching!");

    let discord_state = Arc::new(InMemoryCacheBuilder::new().build());

    // let scheduler_client =
    //     Arc::new(new_scheuler_rpc_client(config.scheduler_rpc_broker_connect_addr.clone()).await?);

    let handle = run_broker(config.common.discord_token.clone(), discord_state.clone()).await?;

    tokio::spawn(dispatch_server::start_server(
        config.broker_rpc_listen_addr.clone(),
        handle,
    ));

    run_http_server(config, discord_state).await;

    Ok(())
}

#[derive(Clone, StructOpt)]
pub struct BrokerConfig {
    #[structopt(flatten)]
    pub(crate) common: common::config::RunConfig,

    #[structopt(
        long,
        env = "BL_BROKER_RPC_LISTEN_ADDR",
        default_value = "127.0.0.1:7480"
    )]
    pub(crate) broker_rpc_listen_addr: String,

    #[structopt(
        long,
        env = "BL_BROKER_HTTP_API_LISTEN_ADDR",
        default_value = "127.0.0.1:7449"
    )]
    pub(crate) http_api_listen_addr: String,
}
