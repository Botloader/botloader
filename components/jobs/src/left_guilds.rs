use std::{sync::Arc, time::Duration};

use common::DiscordConfig;
use dbrokerapi::state_client::ConnectedGuildsResponse;
use stores::Db;
use tracing::{info, warn};
use twilight_http::error::ErrorType;

use crate::{
    job::{Job, JobSpawner, OutputFuture},
    JobsConfig,
};

pub struct LeftGuildSpawner {
    pub db: Db,
    pub conf: JobsConfig,
    pub discord_config: Arc<DiscordConfig>,
}

impl JobSpawner for LeftGuildSpawner {
    fn name(&self) -> &'static str {
        "left_guilds_scanner_remover"
    }

    fn spawn(&self) -> Arc<dyn Job> {
        Arc::new(LeftGuildJob {
            db: self.db.clone(),
            conf: self.conf.clone(),
            discord_config: self.discord_config.clone(),
        })
    }

    fn interval(&self) -> std::time::Duration {
        Duration::from_secs(600)
    }
}

pub struct LeftGuildJob {
    db: Db,
    conf: JobsConfig,
    discord_config: Arc<DiscordConfig>,
}

impl Job for LeftGuildJob {
    fn status(&self) -> String {
        "Running".to_string()
    }

    fn run(self: std::sync::Arc<Self>) -> OutputFuture {
        Box::pin(async move {
            self.scan_for_left_guilds().await?;
            self.delete_left_guilds().await?;

            Ok(())
        })
    }
}

impl LeftGuildJob {
    async fn scan_for_left_guilds(&self) -> Result<(), anyhow::Error> {
        info!("Scanning for left guilds");

        // get a list of all connected guilds
        let client = dbrokerapi::state_client::Client::new(self.conf.broker_api_addr.clone());
        let connected_guilds = match client.get_connected_guilds().await? {
            ConnectedGuildsResponse::NotReady => {
                warn!("broker not ready yet");
                return Ok(());
            }
            ConnectedGuildsResponse::Ready(guilds) => guilds,
        };
        info!("connected guilds: {}", connected_guilds.len());

        let left_guilds = self.db.get_joined_guilds_not_in(&connected_guilds).await?;
        info!("left guilds: {}, {:?}", left_guilds.len(), left_guilds);

        for guild in left_guilds {
            match self.discord_config.client.guild(guild.id).await {
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
                        self.db.set_guild_left_status(guild.id, true).await?;
                    }
                    _ => {
                        // another error occurred
                        return Err(e.into());
                    }
                },
            }
        }

        Ok(())
    }

    async fn delete_left_guilds(&self) -> Result<(), anyhow::Error> {
        info!(
            "Deleting left guilds, min left age days: {}",
            self.conf.delete_guilds_min_left_days
        );

        if self.conf.delete_guilds_min_left_days < 1 {
            panic!("min-age-days needs to be above 0");
        }

        let guilds = self
            .db
            .get_left_guilds(self.conf.delete_guilds_min_left_days as u64 * 24)
            .await?;
        for g in guilds {
            info!("deleting {}", g.id);

            self.db.delete_guild_bucket_store_data(g.id).await?;
            self.db.delete_guild_timer_data(g.id).await?;
            self.db.delete_guild_config_data(g.id).await?;
        }

        Ok(())
    }
}
