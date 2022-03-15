use std::sync::Arc;

use axum::{extract::Extension, http::StatusCode, routing::get, Json, Router};
use clap::Parser;
use stores::{
    config::{ConfigStore, CreateUpdatePremiumSlotBySource, PremiumSlot},
    postgres::Postgres,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::common_init(Some("0.0.0.0:7804"));
    let config = BrokerConfig::parse();

    info!("Launching!");

    let postgres_store = Arc::new(
        Postgres::new_with_url(&config.common.database_url)
            .await
            .unwrap(),
    );

    run_http_server(config, postgres_store).await;

    Ok(())
}

#[derive(Clone, clap::Parser)]
pub struct BrokerConfig {
    #[clap(flatten)]
    pub(crate) common: common::config::RunConfig,

    #[clap(
        long,
        env = "BL_DB_API_HTTP_LISTEN_ADDR",
        default_value = "127.0.0.1:7900"
    )]
    pub(crate) http_api_listen_addr: String,
}

pub async fn run_http_server(conf: crate::BrokerConfig, postgres: Arc<Postgres>) {
    let app = Router::new()
        .route(
            "/create_update_premium_slots_by_source",
            get(handle_post_premium_slots_by_source),
        )
        // .layer(Extension(discord_state))
        .layer(Extension(postgres))
        .layer(axum_metrics_layer::MetricsLayer {
            name: "bl.db.http_api_hits_total",
        });

    let make_service = app.into_make_service();

    // run it with hyper on configured address
    info!("Starting hype on address: {}", conf.http_api_listen_addr);
    let addr = conf.http_api_listen_addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(common::shutdown::wait_shutdown_signal())
        .await
        .unwrap();
}

async fn handle_post_premium_slots_by_source(
    Extension(store): Extension<Arc<Postgres>>,
    Json(body): Json<Vec<CreateUpdatePremiumSlotBySource>>,
) -> Result<Json<Vec<PremiumSlot>>, StatusCode> {
    let mut res = Vec::new();
    for slot in body {
        match store.create_update_premium_slot_by_source(slot).await {
            Ok(v) => res.push(v),
            Err(err) => {
                error!(%err, "failed creating or updating premium slot");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Ok(Json(res))
}
