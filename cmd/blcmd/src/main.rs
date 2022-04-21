use std::{num::NonZeroU64, path::PathBuf};

use clap::{Parser, Subcommand};
use serde::Deserialize;
use stores::config::{ConfigStore, CreateScript};
use stores::postgres::Postgres;
use tracing::info;
use twilight_model::id::Id;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.cmd {
        Command::Login => todo!(),
        Command::Logout => todo!(),
        Command::Status => todo!(),
        Command::Sync => todo!(),
        Command::PushAllChanges => todo!(),
        Command::UpdateScript => todo!(),
        Command::DelScript => todo!(),
        Command::CreateScript => todo!(),
        Command::PublishVersion => todo!(),
        Command::StartDevSession => todo!(),
        Command::StopDevSession => todo!(),
    }

    Ok(())
}

pub async fn load_config() -> anyhow::Result<()> {
    Ok(())
}

#[derive(Clone, clap::Parser)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    Login,
    Logout,
    Status,

    Sync,

    // Guild scripts
    PushAllChanges,
    UpdateScript,
    DelScript,
    CreateScript,

    // Plugins
    PublishVersion,
    StartDevSession,
    StopDevSession,
}

#[derive(Deserialize)]
pub struct Config {
    api_host: String,
    api_token: String,
    api_use_https: bool,
}
