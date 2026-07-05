use std::sync::{Arc, RwLock};

use lru::LruCache;
use stores::Db;
use twilight_model::id::{marker::UserMarker, Id};

use crate::{DiscordOauthClient, TwilightOauthClient};

/// Creates a client for the given user id and access token, used when there's
/// no cached client for the user. Swap this out for a mock factory in tests.
pub type ClientFactory =
    Arc<dyn Fn(Id<UserMarker>, &str) -> Arc<dyn DiscordOauthClient> + Send + Sync>;

type CacheInner = LruCache<Id<UserMarker>, Arc<dyn DiscordOauthClient>>;

#[derive(Clone)]
pub struct ClientCache {
    factory: ClientFactory,
    inner: Arc<RwLock<CacheInner>>,
}

impl ClientCache {
    pub fn new(factory: ClientFactory) -> Self {
        Self {
            factory,
            inner: Arc::new(RwLock::new(LruCache::new(10000))),
        }
    }

    /// A cache producing real discord clients, refreshing tokens through the
    /// provided oauth client and persisting them to the provided db.
    pub fn new_twilight(oauth_client: oauth2::basic::BasicClient, db: Db) -> Self {
        Self::new(Arc::new(move |user_id, access_token| {
            Arc::new(TwilightOauthClient::new(
                user_id,
                access_token.to_owned(),
                oauth_client.clone(),
                db.clone(),
            ))
        }))
    }

    /// Returns the cached client for the user, creating one through the
    /// factory otherwise.
    pub fn fetch(
        &self,
        user_id: Id<UserMarker>,
        access_token: &str,
    ) -> Arc<dyn DiscordOauthClient> {
        let mut write = self.inner.write().unwrap();
        if let Some(v) = write.get(&user_id) {
            return v.clone();
        }

        let client = (self.factory)(user_id, access_token);
        write.put(user_id, client.clone());
        client
    }

    pub fn del(&self, user_id: Id<UserMarker>) {
        let mut write = self.inner.write().unwrap();
        write.pop(&user_id);
    }
}
