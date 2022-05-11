use std::sync::Arc;

use tracing::info;

use twilight_model::{
    oauth::CurrentApplicationInfo,
    user::{CurrentUser, User},
};

#[derive(Debug)]
pub struct DiscordConfig {
    pub bot_user: CurrentUser,
    pub application: CurrentApplicationInfo,
    pub owners: Vec<User>,
    pub client: twilight_http::Client,
}

pub async fn fetch_discord_config(
    token: String,
) -> Result<Arc<DiscordConfig>, twilight_http::Error> {
    let client = twilight_http::Client::new(token);

    // println!("fetching bot and application details from discord...");
    let bot_user = client.current_user().exec().await?.model().await.unwrap();
    info!("discord logged in as: {:?}", bot_user);

    let application = client
        .current_user_application()
        .exec()
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
