use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

mod api_client;

const APP_NAME: &str = "botloadercmd";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = confy::load(APP_NAME)?;

    let ctx = Context {
        _full_args: args.clone(),
        config,
    };

    match args.cmd {
        Command::Login => login(ctx).await?,
        Command::Logout => logout(ctx).await?,
        Command::Status => check_login(ctx).await?,
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

#[derive(Clone, clap::Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clone, Subcommand)]
enum Command {
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

#[derive(Deserialize, Serialize, Clone)]
struct Config {
    api_host: String,
    api_token: String,
    api_use_https: bool,
}

impl Config {
    fn api_client(&self) -> api_client::ApiClient {
        let proto = if self.api_use_https {
            "https://"
        } else {
            "http"
        };
        api_client::ApiClient::new(self.api_token.clone(), format!("{proto}{}", self.api_host))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_host: "api.botloader.io".to_owned(),
            api_token: Default::default(),
            api_use_https: true,
        }
    }
}

struct Context {
    config: Config,
    _full_args: Args,
}

async fn login(ctx: Context) -> anyhow::Result<()> {
    println!("Enter your api token:");

    let stdin = std::io::stdin();
    let mut s = String::new();
    let _ = stdin.read_line(&mut s)?;

    let mut new_config = ctx.config;
    new_config.api_token = s.trim().to_owned();

    // try the token
    let new_client = new_config.api_client();
    let self_user = new_client.get_self_user().await?;

    println!(
        "Logged in as {}#{}!",
        self_user.name,
        self_user.discriminator()
    );

    confy::store(APP_NAME, &new_config)?;

    Ok(())
}

async fn check_login(ctx: Context) -> anyhow::Result<()> {
    if ctx.config.api_token.is_empty() {
        println!("You're not logged in");
        return Ok(());
    }

    let client = ctx.config.api_client();
    let self_user = client.get_self_user().await?;

    println!(
        "Logged in as {}#{}!",
        self_user.name,
        self_user.discriminator()
    );

    Ok(())
}

async fn logout(ctx: Context) -> anyhow::Result<()> {
    if ctx.config.api_token.is_empty() {
        println!("You're not logged in");
        return Ok(());
    }

    let mut new_config = ctx.config;
    new_config.api_token = String::new();
    confy::store(APP_NAME, &new_config)?;

    println!("Removed your login details");
    Ok(())
}
