use std::{num::NonZeroU64, path::PathBuf};

use clap::Parser;
use stores::config::{ConfigStore, CreateScript};
use stores::postgres::Postgres;
use tracing::info;
use twilight_model::id::Id;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RunConfig::parse();
    let guild_id = Id::from(config.guild_id);
    common::init_tracing();

    info!("preparing scripts..");

    let config_store = Postgres::new_with_url(&config.database_url).await?;

    let compiled_filter_regex = config.filter.map(|v| regex::Regex::new(&v).unwrap());

    let dir = std::fs::read_dir(config.scripts_path)?;
    for entry in dir {
        let unwrapped = entry.unwrap();
        let os_name = unwrapped.file_name();
        let name_with_suffix = os_name.to_str().unwrap();
        if !name_with_suffix.ends_with(".ts") || name_with_suffix.ends_with(".d.ts") {
            continue;
        }

        if name_with_suffix != "lib.ts" {
            if let Some(filter) = &compiled_filter_regex {
                if !filter.is_match(name_with_suffix) {
                    info!("filtering active, skipped test {}", name_with_suffix);
                    continue;
                }
            }
        }

        let contents = std::fs::read_to_string(unwrapped.path())?;

        config_store
            .create_script(
                guild_id,
                CreateScript {
                    enabled: true,
                    name: name_with_suffix.strip_suffix(".ts").unwrap().to_string(),
                    original_source: contents,
                    plugin_auto_update: None,
                    plugin_id: None,
                },
            )
            .await?;

        info!("added scrpt {}", name_with_suffix);
    }

    Ok(())
}

#[derive(Clone, clap::Parser)]
pub struct RunConfig {
    #[clap(long, env = "DATABASE_URL")]
    database_url: String,

    #[clap(long)]
    scripts_path: PathBuf,

    #[clap(long)]
    guild_id: NonZeroU64,

    #[clap(long)]
    filter: Option<String>,
}
