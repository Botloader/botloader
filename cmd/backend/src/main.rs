use std::env::current_exe;

use clap::{Parser, Subcommand};
use discordbroker::BrokerConfig;
use jobs::JobsConfig;
use scheduler::{SchedulerConfig, WorkerLaunchConfig};
use vmworker::WorkerConfig;
use webapi::WebConfig;

#[tokio::main]
async fn main() {
    let config: Cli = common::load_config();
    match config.command {
        Commands::WebApi(conf) => webapi::run(conf.common, conf.web_config, true).await,
        Commands::DiscordBroker(conf) => {
            discordbroker::run(conf.common, conf.broker_config, true).await;
        }
        Commands::Scheduler(conf) => {
            scheduler::run(
                conf.common,
                conf.scheduler_config,
                true,
                get_worker_launch_config(),
            )
            .await
        }
        Commands::VmWorker(conf) => vmworker::run(conf).await.unwrap(),
        Commands::Full(full_conf) => {
            common::setup_tracing(&full_conf.common, "full");
            common::setup_metrics("0.0.0.0:7801");
            tokio::spawn(discordbroker::run(
                full_conf.common.clone(),
                full_conf.broker_config,
                false,
            ));
            tokio::spawn(webapi::run(
                full_conf.common.clone(),
                full_conf.web_config,
                false,
            ));
            scheduler::run(
                full_conf.common.clone(),
                full_conf.scheduler_config,
                false,
                get_worker_launch_config(),
            )
            .await;
        }
        Commands::Jobs(conf) => jobs::run(conf.common, conf.jobs_config, true).await,
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    WebApi(WrappedWebConfig),
    DiscordBroker(WrappedBrokerConfig),
    Scheduler(WrappedSchedulerConfig),
    VmWorker(WorkerConfig),
    Jobs(WrappedJobsConfig),
    Full(Box<AllConfig>),
}

fn get_worker_launch_config() -> WorkerLaunchConfig {
    let current_bin = current_exe().unwrap();
    return WorkerLaunchConfig {
        cmd: current_bin.to_str().unwrap().to_string(),
        args: vec!["vm-worker".to_string()],
    };
}

#[derive(Clone, clap::Parser)]
struct AllConfig {
    #[clap(flatten)]
    common: common::config::RunConfig,

    #[clap(flatten)]
    web_config: WebConfig,

    #[clap(flatten)]
    broker_config: BrokerConfig,

    #[clap(flatten)]
    scheduler_config: SchedulerConfig,
}

#[derive(Clone, clap::Parser, Debug)]
struct WrappedWebConfig {
    #[clap(flatten)]
    common: common::config::RunConfig,

    #[clap(flatten)]
    web_config: WebConfig,
}

#[derive(Clone, clap::Parser, Debug)]
struct WrappedBrokerConfig {
    #[clap(flatten)]
    common: common::config::RunConfig,

    #[clap(flatten)]
    broker_config: BrokerConfig,
}

#[derive(Clone, clap::Parser, Debug)]
struct WrappedSchedulerConfig {
    #[clap(flatten)]
    common: common::config::RunConfig,

    #[clap(flatten)]
    scheduler_config: SchedulerConfig,
}

#[derive(Clone, clap::Parser, Debug)]
struct WrappedJobsConfig {
    #[clap(flatten)]
    common: common::config::RunConfig,

    #[clap(flatten)]
    jobs_config: JobsConfig,
}
