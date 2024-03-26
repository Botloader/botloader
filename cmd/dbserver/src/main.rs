use axum::{extract::Extension, http::StatusCode, routing::get, Json, Router};
use stores::{
    config::{CreateUpdatePremiumSlotBySource, PremiumSlot},
    Db,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: BrokerConfig = common::load_config();
    common::setup_tracing(&config.common, "dbserver");
    common::setup_metrics("0.0.0.0:7804");

    info!("Launching!");

    let postgres_store = Db::new_with_url(&config.common.database_url).await.unwrap();

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

pub async fn run_http_server(conf: crate::BrokerConfig, postgres: Db) {
    let app = Router::new()
        .route(
            "/create_update_premium_slots_by_source",
            get(handle_post_premium_slots_by_source),
        )
        // .layer(Extension(discord_state))
        .layer(Extension(postgres))
        .layer(axum_metrics_layer::MetricsLayer {
            name_prefix: "bl.db",
        });

    // run it with hyper on configured address
    info!("Starting hype on address: {}", conf.http_api_listen_addr);

    let listener = tokio::net::TcpListener::bind(conf.http_api_listen_addr)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_post_premium_slots_by_source(
    Extension(store): Extension<Db>,
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
