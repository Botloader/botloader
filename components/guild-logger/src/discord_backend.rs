use std::sync::Arc;

use crate::{LogEntry, LogLevel};
use common::DiscordConfig;
use stores::Db;
use tracing::error;

pub struct DiscordLogger {
    discord_config: Arc<DiscordConfig>,
    db: Db,
}

impl DiscordLogger {
    pub fn new(discord_config: Arc<DiscordConfig>, db: Db) -> Self {
        Self { db, discord_config }
    }
}

#[async_trait::async_trait]
impl crate::GuildLoggerBackend for DiscordLogger {
    async fn handle_entry(&self, entry: LogEntry) {
        let guild_id = entry.guild_id;
        let message = match format_entry(entry) {
            Some(msg) => msg,
            None => return,
        };

        let conf = match self.db.get_guild_meta_config_or_default(guild_id).await {
            Ok(v) => v,
            Err(err) => {
                error!(%err, "failed fetching config for guild logging");
                return;
            }
        };

        if let Some(channel_id) = conf.error_channel_id {
            if let Ok(next) = self
                .discord_config
                .client
                .create_message(channel_id)
                .content(&message)
            {
                next.await.ok();
            }
        }
    }
}

fn format_entry(entry: LogEntry) -> Option<String> {
    if matches!(entry.level, LogLevel::Error | LogLevel::Critical) {
        let prefix = if let Some(script_ctx) = entry.script_context {
            format!("[{} {}]", entry.level, script_ctx)
        } else {
            format!("[{}]", entry.level)
        };
        Some(format!("{}: {}", prefix, entry.message))
    } else {
        None
    }
}
