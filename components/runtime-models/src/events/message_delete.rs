use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/MessageDelete.ts")]
#[serde(rename_all = "camelCase")]
pub struct MessageDelete {
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub id: String,
}

impl From<twilight_model::gateway::payload::incoming::MessageDelete> for MessageDelete {
    fn from(v: twilight_model::gateway::payload::incoming::MessageDelete) -> Self {
        Self {
            channel_id: v.channel_id.to_string(),
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
        }
    }
}
