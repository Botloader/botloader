use std::sync::Arc;

use tracing::info;

use twilight_model::{
    oauth::Application,
    user::{CurrentUser, User},
};

#[derive(Debug)]
pub struct DiscordConfig {
    pub bot_user: CurrentUser,
    pub application: Application,
    pub owners: Vec<User>,
    pub client: twilight_http::Client,
}

pub async fn fetch_discord_config(
    token: String,
) -> Result<Arc<DiscordConfig>, twilight_http::Error> {
    // Needed because twilight does not do this and that causes a panic down the line when issuing requests to discord
    // The error can be ignored because this function is called multiple types in full mode
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let client = twilight_http::Client::new(token);

    // println!("fetching bot and application details from discord...");
    let bot_user = client.current_user().await?.model().await.unwrap();
    info!("discord logged in as: {:?}", bot_user);

    let application = client
        .current_user_application()
        .await?
        .model()
        .await
        .unwrap();
    info!("discord application: {:?}", application.name);

    let owners = match &application.team {
        Some(t) => t.members.iter().map(|e| e.user.clone()).collect(),
        None => {
            if let Some(owner) = &application.owner {
                vec![owner.clone()]
            } else {
                Vec::new()
            }
        }
    };

    info!(
        "discord application owners: {:?}",
        owners.iter().map(|o| o.id).collect::<Vec<_>>()
    );

    Ok(Arc::new(DiscordConfig {
        application,
        bot_user,
        owners,
        client,
    }))
}

impl DiscordConfig {
    pub fn interaction_client(&self) -> twilight_http::client::InteractionClient<'_> {
        self.client.interaction(self.application.id)
    }
}
