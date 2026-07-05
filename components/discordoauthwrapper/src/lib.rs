use twilight_model::user::{CurrentUser, CurrentUserGuild};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

mod cache;
mod twilight_client;
pub use cache::{ClientCache, ClientFactory};
pub use twilight_client::TwilightOauthClient;

/// Discord api client operating on behalf of a single user through oauth
/// tokens, refreshing them as needed.
#[async_trait::async_trait]
pub trait DiscordOauthClient: Send + Sync {
    async fn current_user(&self) -> Result<CurrentUser, BoxError>;
    async fn current_user_guilds(&self) -> Result<Vec<CurrentUserGuild>, BoxError>;

    /// set when the refresh token is no longer valid, meaning the user has to
    /// re-authorize and all their sessions should be discarded
    fn is_broken(&self) -> bool;
}
