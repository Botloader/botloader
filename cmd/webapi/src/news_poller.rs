use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use serde::Serialize;
use tracing::{error, info};
use twilight_model::{
    channel::Channel,
    id::{
        marker::{ChannelMarker, GuildMarker, MessageMarker},
        Id,
    },
};

pub struct NewsPoller {
    discord_http: Arc<twilight_http::Client>,
    follow_channels: Vec<Id<ChannelMarker>>,
    guild_channels: Vec<Channel>,
    items: Arc<RwLock<Arc<Vec<NewsItem>>>>,
}

impl NewsPoller {
    pub async fn new(
        discord_http: Arc<twilight_http::Client>,
        follow_channels: Vec<Id<ChannelMarker>>,
        guild_id: Id<GuildMarker>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let channels = discord_http
            .guild_channels(guild_id)
            .await?
            .models()
            .await?;

        Ok(Self {
            discord_http,
            follow_channels,
            guild_channels: channels,
            items: Arc::new(RwLock::new(Arc::new(Vec::new()))),
        })
    }

    pub fn handle(&self) -> NewsHandle {
        NewsHandle {
            items: self.items.clone(),
        }
    }

    pub async fn run(self) {
        loop {
            info!(
                follow_channels = self.follow_channels.len(),
                "fetching news"
            );

            match self.fetch().await {
                Ok(items) => {
                    let mut handle = self.items.write().unwrap();
                    *handle = Arc::new(items);
                }
                Err(err) => {
                    error!(%err, "failed fetching news items");
                }
            }

            tokio::time::sleep(Duration::from_secs(300)).await
        }
    }

    async fn fetch(&self) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_msgs = Vec::new();

        for channel in &self.follow_channels {
            let mut msgs = self
                .discord_http
                .channel_messages(*channel)
                .limit(100)
                .unwrap()
                .await?
                .models()
                .await?;

            all_msgs.append(&mut msgs);
        }

        all_msgs.sort_by(|a, b| a.id.get().cmp(&b.id.get()).reverse());

        Ok(all_msgs
            .into_iter()
            .map(|v| {
                let channel = self
                    .guild_channels
                    .iter()
                    .find(|cv| cv.id == v.channel_id)
                    .map(|cv| cv.name.as_deref().unwrap_or("").to_owned())
                    .unwrap_or_else(|| ToString::to_string(&v.channel_id));

                NewsItem {
                    author: NewsAuthor {
                        avatar_url: v.author.avatar.map(|hash| {
                            format!(
                                "https://cdn.discordapp.com/avatars/{}/{}.png?size=256",
                                v.author.id, hash
                            )
                        }),
                        username: v.author.name,
                    },
                    channel_id: v.channel_id,
                    channel_name: channel,
                    content: v.content,
                    message_id: v.id,
                    posted_at: v.timestamp.as_secs() * 1000,
                }
            })
            .collect())
    }
}

#[derive(Clone, Default)]
pub struct NewsHandle {
    items: Arc<RwLock<Arc<Vec<NewsItem>>>>,
}

impl NewsHandle {
    pub fn get_items(&self) -> Arc<Vec<NewsItem>> {
        let handle = self.items.read().unwrap();
        (*handle).clone()
    }
}

#[derive(Serialize)]
pub struct NewsItem {
    author: NewsAuthor,
    message_id: Id<MessageMarker>,
    channel_id: Id<ChannelMarker>,
    channel_name: String,
    content: String,
    posted_at: i64,
}

#[derive(Serialize)]
pub struct NewsAuthor {
    username: String,
    avatar_url: Option<String>,
}
