use std::{net::SocketAddr, str::FromStr};

use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    EnvFilter,
};

pub mod config;
pub mod discord;
pub mod plugin;
pub mod shutdown;
pub mod user;

pub use discord::*;

pub fn common_init(metrics_listen_addr: Option<&str>) {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(dotenv::Error::Io(_)) => {} // ignore io errors
        Err(e) => panic!("failed loading dotenv file: {e}"),
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
        .with_span_events(FmtSpan::NONE);
    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
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
