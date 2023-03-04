use clap::Parser;
use common::{shutdown, DiscordConfig};
use dbrokerapi::state_client::ConnectedGuildsResponse;
use stores::{
    bucketstore::BucketStore, config::ConfigStore, postgres::Postgres, timers::TimerStore,
};
use tracing::{error, info, warn, Instrument};
use twilight_http::error::ErrorType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::common_init(None);
    let config = Config::parse();

    let discord_config = common::fetch_discord_config(config.common.discord_token.clone())
        .await
        .expect("failed fetching discord config");

    let db = Postgres::new_with_url(&config.common.database_url)
        .await
        .unwrap();

    let stop_future = shutdown::wait_shutdown_signal();
    tokio::pin!(stop_future);
    loop {
        let span = tracing::info_span!("bl_jobs");

        let config_clone = config.clone();
        let db_clone = db.clone();
        let discord_config_clone = discord_config.clone();
        async move {
            if let Err(err) =
                scan_for_left_guilds(&config_clone, &db_clone, &discord_config_clone).await
            {
                error!(err, "failed scanning for left guilds");
            }
            if let Err(err) = delete_left_guilds(&config_clone, &db_clone).await {
                error!(err, "failed deleting left guilds");
            }
        }
        .instrument(span)
        .await;

        tokio::select! {
            _ = &mut stop_future => {
                break;
            },
            _ = tokio::time::sleep(std::time::Duration::from_secs(60*60)) => {
                continue;
            }
        }
    }

    Ok(())

    // match &config.command {
    //     Command::ScanForLeftGuilds => scan_for_left_guilds(config, db, discord_config).await,
    //     Command::DeleteLeftGuilds(opts) => {
    //         delete_left_guilds(config.clone(), opts.clone(), db).await
    //     }
    // }
}

#[derive(Clone, clap::Parser)]
struct Config {
    #[clap(flatten)]
    common: common::config::RunConfig,

    #[clap(
        long,
        env = "BL_BROKER_API_ADDR",
        default_value = "http://0.0.0.0:7449"
    )]
    broker_api_addr: String,

    #[clap(long, env = "BL_JOBS_DELETE_GUILDS_MIN_LEFT_DAYS", default_value = "7")]
    delete_guilds_min_left_days: u16,
}

async fn scan_for_left_guilds(
    conf: &Config,
    db: &Postgres,
    discord_config: &DiscordConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Scanning for left guilds");

    // get a list of all connected guilds
    let client = dbrokerapi::state_client::Client::new(conf.broker_api_addr.clone());
    let connected_guilds = match client.get_connected_guilds().await? {
        ConnectedGuildsResponse::NotReady => {
            warn!("broker not ready yet");
            return Ok(());
        }
        ConnectedGuildsResponse::Ready(guilds) => guilds,
    };
    info!("connected guilds: {}", connected_guilds.len());

    let left_guilds = db.get_joined_guilds_not_in(&connected_guilds).await?;
    info!("left guilds: {}, {:?}", left_guilds.len(), left_guilds);

    for guild in left_guilds {
        match discord_config.client.guild(guild.id).await {
            Ok(_) => {
                // still connected
                info!("still connected to {}, skipping", guild.id);
                continue;
            }
            Err(e) => match e.kind() {
                ErrorType::Response {
                    body: _,
                    error: _,
                    status,
                } if status.get() == 403 => {
                    info!("marking guild as left: {}", guild.id);
                    db.set_guild_left_status(guild.id, true).await?;
                }
                _ => {
                    // another error occured
                    return Err(Box::new(e));
                }
            },
        }
    }

    Ok(())
}

async fn delete_left_guilds(
    conf: &Config,
    db: &Postgres,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Deleting left guilds, min left age days: {}",
        conf.delete_guilds_min_left_days
    );

    if conf.delete_guilds_min_left_days < 1 {
        panic!("min-age-days needs to be above 0");
    }

    let guilds = db
        .get_left_guilds(conf.delete_guilds_min_left_days as u64 * 24)
        .await?;
    for g in guilds {
        info!("deleting {}", g.id);

        db.delete_guild_bucket_store_data(g.id).await?;
        db.delete_guild_timer_data(g.id).await?;
        db.delete_guild_config_data(g.id).await?;
    }

    Ok(())
}
