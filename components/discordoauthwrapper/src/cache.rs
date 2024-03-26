use std::{
    fmt::{Debug, Display},
    sync::{Arc, RwLock},
};

use lru::LruCache;
use twilight_model::id::{marker::UserMarker, Id};

use crate::DiscordOauthApiClient;

type CacheInner<T, TU> = LruCache<Id<UserMarker>, DiscordOauthApiClient<T, TU>>;

pub struct ClientCache<T, TU> {
    inner: Arc<RwLock<CacheInner<T, TU>>>,
}

impl<T, TU> Clone for ClientCache<T, TU> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, TU> Default for ClientCache<T, TU> {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner::new(10000))),
        }
    }
}

impl<T, TU> ClientCache<T, TU>
where
    T: crate::DiscordOauthApiProvider + 'static,
    TU: crate::TokenRefresher + 'static,
    T::OtherError: Debug + Display + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner::new(10000))),
        }
    }

    pub fn get(&self, user_id: Id<UserMarker>) -> Option<DiscordOauthApiClient<T, TU>> {
        let mut write = self.inner.write().unwrap();
        let client = write.get(&user_id);
        client.cloned()
    }

    pub fn fetch<F, FR>(
        &self,
        user_id: Id<UserMarker>,
        f: F,
    ) -> Result<DiscordOauthApiClient<T, TU>, FR>
    where
        F: FnOnce() -> Result<DiscordOauthApiClient<T, TU>, FR>,
    {
        let mut write = self.inner.write().unwrap();
        if let Some(v) = write.get(&user_id) {
            return Ok(v.clone());
        }

        match f() {
            Ok(v) => {
                write.put(user_id, v.clone());
                Ok(v)
            }
            Err(err) => Err(err),
        }
    }

    pub fn del(&self, user_id: Id<UserMarker>) {
        let mut write = self.inner.write().unwrap();
        write.pop(&user_id);
    }
}
