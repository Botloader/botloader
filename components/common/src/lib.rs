use std::{net::SocketAddr, str::FromStr, sync::Arc};

use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    EnvFilter,
};
use twilight_model::{
    oauth::CurrentApplicationInfo,
    user::{CurrentUser, User},
};

pub mod config;
pub mod shutdown;

pub fn common_init(metrics_listen_addr: Option<&str>) {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(dotenv::Error::Io(_)) => {} // ignore io errors
        Err(e) => panic!("failed loading dotenv file: {}", e),
    }
    init_tracing();

    if let Some(metrics_listen_addr) = metrics_listen_addr {
        let (recorder, exporter) = PrometheusBuilder::new()
            .with_http_listener(SocketAddr::from_str(metrics_listen_addr).unwrap())
            .build()
            .unwrap();

        metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

        info!("exposing metrics at {}", metrics_listen_addr);
        tokio::spawn(exporter);
    }
}

pub fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
}

#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub bot_user: CurrentUser,
    pub application: CurrentApplicationInfo,
    pub owners: Vec<User>,
    pub client: Arc<twilight_http::Client>,
}

pub async fn fetch_discord_config(token: String) -> Result<DiscordConfig, twilight_http::Error> {
    let client = twilight_http::Client::new(token);

    // println!("fetching bot and application details from discord...");
    let bot_user = client.current_user().exec().await?.model().await.unwrap();
    info!("discord logged in as: {:?}", bot_user);

    let application = client
        .current_user_application()
        .exec()
        .await?
        .model()
        .await
        .unwrap();
    info!("discord application: {:?}", application.name);

    let owners = match &application.team {
        Some(t) => t.members.iter().map(|e| e.user.clone()).collect(),
        None => vec![application.owner.clone()],
    };
    info!(
        "discord application owners: {:?}",
        owners.iter().map(|o| o.id).collect::<Vec<_>>()
    );

    client.set_application_id(application.id);

    Ok(DiscordConfig {
        application,
        bot_user,
        owners,
        client: Arc::new(client),
    })
}

// #[derive(Clone)]
// struct MetricsLabelFilter;

// impl LabelFilter for MetricsLabelFilter {
//     fn should_include_label(&self, label: &metrics::Label) -> bool {
//         if label.key() == "script_id" {
//             return false;
//         }
//         true
//     }
// }
