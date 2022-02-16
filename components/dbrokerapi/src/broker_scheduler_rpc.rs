use serde::{Deserialize, Serialize};
use twilight_model::{
    gateway::event::{DispatchEvent, DispatchEventWithTypeDeserializer},
    id::{marker::GuildMarker, Id},
};

use serde::de::DeserializeSeed;

pub struct GuildEvent {
    pub guild_id: Id<GuildMarker>,
    pub t: String,
    pub event: Box<DispatchEvent>,
}

impl TryFrom<RawDiscordEvent> for GuildEvent {
    type Error = serde_json::Error;

    fn try_from(value: RawDiscordEvent) -> Result<Self, Self::Error> {
        let deserializer = DispatchEventWithTypeDeserializer::new(&value.t);
        let event: DispatchEvent = deserializer.deserialize(value.event)?;

        Ok(GuildEvent {
            event: Box::new(event),
            guild_id: value.guild_id,
            t: value.t,
        })
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct RawDiscordEvent {
    pub guild_id: Id<GuildMarker>,
    pub t: String,
    pub event: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloData {
    pub connected_guilds: Vec<Id<GuildMarker>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerEvent {
    Hello(HelloData),
    DiscordEvent(RawDiscordEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SchedulerEvent {
    Ack,
}
