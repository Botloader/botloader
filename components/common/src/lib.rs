use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use metrics_exporter_prometheus::PrometheusBuilder;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::{info, Level};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod discord;
pub mod plugin;
pub mod shutdown;
pub mod user;

pub use discord::*;

pub fn load_config<T: Parser>() -> T {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(dotenv::Error::Io(_)) => {} // ignore io errors
        Err(e) => panic!("failed loading dotenv file: {e}"),
    }

    T::parse()
}

pub fn setup_metrics(metrics_listen_addr: &str) {
    let (recorder, exporter) = PrometheusBuilder::new()
        .with_http_listener(SocketAddr::from_str(metrics_listen_addr).unwrap())
        .build()
        .unwrap();

    metrics::set_global_recorder(recorder).unwrap();

    info!("exposing metrics at {}", metrics_listen_addr);
    tokio::spawn(exporter);
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

pub fn setup_tracing(config: &config::RunConfig, service_name: &str) {
    if let Some(otlp_url) = config.otlp_grpc_url.as_ref() {
        setup_tracing_otlp(otlp_url, service_name.to_owned())
    } else {
        setup_tracing_stdout()
    }
}

pub fn setup_tracing_otlp(url: &str, service_name: String) {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(url);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![KeyValue::new("service.name", service_name)]),
        ))
        .install_simple()
        .expect("valid tracing config");

    let otlp_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // We need to register our layer with `tracing`.
    tracing_subscriber::registry()
        .with(global_filters().with_default(Level::INFO))
        .with(otlp_layer)
        // Tracing error sadly can't be used as it results in deadlocks
        .with(tracing_subscriber::fmt::Layer::new())
        .init();
}

pub fn setup_tracing_stdout() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_span_events(FmtSpan::NONE);
    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(global_filters())
        .with(fmt_layer)
        .with(env_filter)
        .init();
}

fn global_filters() -> Targets {
    Targets::new()
        .with_default(Level::DEBUG)
        .with_target("swc", Level::ERROR)
}
