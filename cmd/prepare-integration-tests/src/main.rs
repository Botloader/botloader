use std::ops::Add;
use std::{num::NonZeroU64, path::PathBuf};

use runtime_models::internal::script::SettingsOptionValue;
use stores::config::{CreateScript, CreateUpdatePremiumSlotBySource, UpdateScript};
use stores::Db;
use tracing::info;
use twilight_model::id::Id;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: RunConfig = common::load_config();
    common::setup_tracing_stdout();

    let guild_id = Id::from(config.guild_id);

    info!("preparing scripts..");

    let config_store = Db::new_with_url(&config.database_url).await?;
    let premium_slot: stores::config::PremiumSlot = config_store
        .create_update_premium_slot_by_source(CreateUpdatePremiumSlotBySource {
            expires_at: chrono::Utc::now().add(chrono::Duration::days(100)),
            manage_url: String::new(),
            message: "testing".to_owned(),
            source: "testing".to_owned(),
            source_id: "testing".to_owned(),
            title: "testing".to_owned(),
            state: stores::config::PremiumSlotState::Active,
            tier: stores::config::PremiumSlotTier::Premium,
            user_id: Some(Id::new(1)),
        })
        .await
        .unwrap();
    config_store
        .update_premium_slot_attachment(Id::new(1), premium_slot.id, Some(guild_id))
        .await
        .unwrap();

    let compiled_filter_regex = config.filter.map(|v| regex::Regex::new(&v).unwrap());

    let dir = std::fs::read_dir(config.scripts_path)?;
    let entries = dir.collect::<Result<Vec<_>, _>>().unwrap();

    for entry in &entries {
        let os_name = entry.file_name();
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

        let contents = std::fs::read_to_string(entry.path())?;

        let name_without_suffix = name_with_suffix.strip_suffix(".ts").unwrap();
        let added_script = config_store
            .create_script(
                guild_id,
                CreateScript {
                    enabled: true,
                    name: name_without_suffix.to_string(),
                    original_source: contents,
                    plugin_auto_update: None,
                    plugin_id: None,
                    plugin_version_number: None,
                },
            )
            .await?;

        info!("added script {} id {}", name_with_suffix, added_script.id);

        let settings_file_name = format!("{}.settings.json", name_without_suffix);
        if let Some(settings_entry) = entries
            .iter()
            .find(|v| v.file_name().to_str().unwrap() == settings_file_name)
        {
            let contents = std::fs::read_to_string(settings_entry.path())?;
            let deserialized: Vec<SettingsOptionValue> = serde_json::from_str(&contents)?;
            config_store
                .update_script(
                    guild_id,
                    UpdateScript {
                        id: added_script.id,
                        name: None,
                        original_source: None,
                        enabled: None,
                        contributes: None,
                        plugin_version_number: None,
                        settings_definitions: None,
                        settings_values: Some(deserialized),
                    },
                )
                .await?;

            info!("loaded script settings {}", settings_file_name);
        }
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
