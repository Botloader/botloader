use std::{collections::HashMap, sync::RwLock};

use crate::LogEntry;
use tokio::sync::broadcast::{self, Receiver};
use twilight_model::id::{marker::GuildMarker, Id};

#[derive(Default)]
pub struct GuildSubscriberBackend {
    subscriptions: RwLock<HashMap<Id<GuildMarker>, broadcast::Sender<LogEntry>>>,
}

impl GuildSubscriberBackend {
    pub fn subscribe(&self, guild_id: Id<GuildMarker>) -> Receiver<LogEntry> {
        let mut subs = self.subscriptions.write().unwrap();
        if let Some(entry) = subs.get(&guild_id) {
            entry.subscribe()
        } else {
            let (sender, receiver) = broadcast::channel(1000);
            subs.insert(guild_id, sender);
            receiver
        }
    }
}
#[async_trait::async_trait]
impl crate::GuildLoggerBackend for GuildSubscriberBackend {
    async fn handle_entry(&self, entry: LogEntry) {
        let guild_id = entry.guild_id;
        {
            // fast path with read lock
            let read = self.subscriptions.read().unwrap();
            let sender = match read.get(&guild_id) {
                Some(v) => v,
                None => return,
            };

            if sender.send(entry.clone()).is_ok() {
                return;
            }
        };

        // the fast path failed, we need a full write lock
        // to potentially remove a entry from this
        let mut write = self.subscriptions.write().unwrap();
        let sender = match write.get(&guild_id) {
            Some(v) => v,
            None => return,
        };

        // try sending it again, things might have changed while we upgraded
        // the lock
        if sender.send(entry).is_ok() {
            return;
        }

        // we need to remove the subscriptions, it has no more receivers
        write.remove_entry(&guild_id);
    }
}
