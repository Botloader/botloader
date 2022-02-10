use std::{
    fmt::{Debug, Display},
    sync::{Arc, RwLock},
};

use lru::LruCache;
use twilight_model::id::UserId;

use crate::DiscordOauthApiClient;

type CacheInner<T, TU, ST> = LruCache<UserId, DiscordOauthApiClient<T, TU, ST>>;

pub struct ClientCache<T, TU, ST> {
    inner: Arc<RwLock<CacheInner<T, TU, ST>>>,
}

impl<T, TU, ST> Clone for ClientCache<T, TU, ST> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, TU, ST> Default for ClientCache<T, TU, ST> {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner::new(10000))),
        }
    }
}

impl<T, TU, ST> ClientCache<T, TU, ST>
where
    T: crate::DiscordOauthApiProvider + 'static,
    TU: crate::TokenRefresher + 'static,
    ST: crate::SessionStore + 'static,
    T::OtherError: Debug + Display + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner::new(10000))),
        }
    }

    pub fn get(&self, user_id: UserId) -> Option<DiscordOauthApiClient<T, TU, ST>> {
        let mut write = self.inner.write().unwrap();
        let client = write.get(&user_id);
        client.cloned()
    }

    pub fn fetch<F, FR>(
        &self,
        user_id: UserId,
        f: F,
    ) -> Result<DiscordOauthApiClient<T, TU, ST>, FR>
    where
        F: FnOnce() -> Result<DiscordOauthApiClient<T, TU, ST>, FR>,
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

    pub fn del(&self, user_id: UserId) {
        let mut write = self.inner.write().unwrap();
        write.pop(&user_id);
    }
}
