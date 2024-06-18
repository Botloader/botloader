use common::shutdown;
use stores::Db;

mod job;
mod left_guilds;
mod plugin_stats;

pub async fn run(
    common_conf: common::config::RunConfig,
    config: JobsConfig,
    setup_metrics_and_tracing: bool,
) {
    if setup_metrics_and_tracing {
        // common::setup_tracing(&common_conf, "scheduler");
        // common::setup_metrics("0.0.0.0:7803");

        common::setup_tracing(&common_conf, "jobs");
    }

    // common::setup_metrics("0.0.0.0:7802");

    let discord_config = common::fetch_discord_config(common_conf.discord_token.clone())
        .await
        .expect("failed fetching discord config");

    let db = Db::new_with_url(&common_conf.database_url).await.unwrap();

    let stop_future = shutdown::wait_shutdown_signal();

    job::JobRunner::new_run(
        vec![
            Box::new(left_guilds::LeftGuildSpawner {
                conf: config.clone(),
                db: db.clone(),
                discord_config: discord_config.clone(),
            }),
            Box::new(plugin_stats::PluginStatsJobSpawner { db: db.clone() }),
        ],
        stop_future,
    )
    .await;
}

#[derive(Clone, clap::Parser, Debug)]
pub struct JobsConfig {
    #[clap(
        long,
        env = "BL_BROKER_API_ADDR",
        default_value = "http://0.0.0.0:7449"
    )]
    broker_api_addr: String,

    #[clap(long, env = "BL_JOBS_DELETE_GUILDS_MIN_LEFT_DAYS", default_value = "7")]
    delete_guilds_min_left_days: u16,
}
