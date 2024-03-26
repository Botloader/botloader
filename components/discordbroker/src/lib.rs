use std::sync::{atomic::AtomicBool, Arc};

use stores::Db;
use tracing::info;
use twilight_cache_inmemory::InMemoryCacheBuilder;

use crate::{broker::run_broker, http_api::run_http_server};

mod broker;
mod dispatch_server;
mod http_api;

pub async fn run(
    common_conf: common::config::RunConfig,
    config: BrokerConfig,
    setup_metrics_and_tracing: bool,
) {
    if setup_metrics_and_tracing {
        common::setup_tracing(&common_conf, "discordbroker");
        common::setup_metrics("0.0.0.0:7802");
    }

    info!("Launching broker");

    let discord_state = Arc::new(InMemoryCacheBuilder::new().build());

    // let scheduler_client =
    //     Arc::new(new_scheuler_rpc_client(config.scheduler_rpc_broker_connect_addr.clone()).await?);

    let postgres_store = Db::new_with_url(&common_conf.database_url).await.unwrap();

    let ready = Arc::new(AtomicBool::new(false));
    let handle = run_broker(
        common_conf.discord_token.clone(),
        discord_state.clone(),
        postgres_store,
        ready.clone(),
    )
    .await
    .unwrap();

    tokio::spawn(dispatch_server::start_server(
        config.broker_rpc_listen_addr.clone(),
        handle.clone(),
    ));

    run_http_server(config, discord_state, ready.clone(), handle).await;

    // Ok(())
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let config: BrokerConfig = common::load_config();
//     common::setup_tracing(&config.common, "discordbroker");
//     common::setup_metrics("0.0.0.0:7802");

//     // let discord_config = common::fetch_discord_config(config.common.discord_token.clone())
//     //     .await
//     //     .expect("failed fetching discord config");

//     info!("Launching!");

//     let discord_state = Arc::new(InMemoryCacheBuilder::new().build());

//     // let scheduler_client =
//     //     Arc::new(new_scheuler_rpc_client(config.scheduler_rpc_broker_connect_addr.clone()).await?);

//     let postgres_store = Arc::new(
//         Postgres::new_with_url(&config.common.database_url)
//             .await
//             .unwrap(),
//     );

//     let ready = Arc::new(AtomicBool::new(false));
//     let handle = run_broker(
//         config.common.discord_token.clone(),
//         discord_state.clone(),
//         postgres_store,
//         ready.clone(),
//     )
//     .await?;

//     tokio::spawn(dispatch_server::start_server(
//         config.broker_rpc_listen_addr.clone(),
//         handle.clone(),
//     ));
//     run_http_server(config, discord_state, ready.clone(), handle).await;

//     Ok(())
// }

#[derive(Clone, clap::Parser, Debug)]
pub struct BrokerConfig {
    #[clap(
        long,
        env = "BL_BROKER_RPC_LISTEN_ADDR",
        default_value = "127.0.0.1:7480"
    )]
    pub(crate) broker_rpc_listen_addr: String,

    #[clap(
        long,
        env = "BL_BROKER_HTTP_API_LISTEN_ADDR",
        default_value = "127.0.0.1:7449"
    )]
    pub(crate) http_api_listen_addr: String,
}
