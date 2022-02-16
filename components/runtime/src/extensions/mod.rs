use twilight_model::{
    channel::GuildChannel,
    id::{marker::GenericMarker, Id},
};
use vm::AnyError;

use crate::RuntimeContext;

pub mod console;
pub mod discord;
pub mod httpclient;
pub mod storage;
pub mod tasks;

// ensures the provided channel is in the guild, also checking the api as fallback
pub(crate) async fn get_guild_channel(
    rt_ctx: &RuntimeContext,
    channel_id_str: &str,
) -> Result<GuildChannel, AnyError> {
    let channel_id = if let Some(channel_id) = Id::new_checked(channel_id_str.parse()?) {
        channel_id
    } else {
        return Err(anyhow::anyhow!("invalid channel id"));
    };

    match rt_ctx
        .bot_state
        .get_channel(rt_ctx.guild_id, channel_id)
        .await?
    {
        Some(c) => {
            if c.guild_id() != Some(rt_ctx.guild_id) {
                Err(anyhow::anyhow!("Unknown channel"))
            } else {
                Ok(c)
            }
        }
        None => {
            let channel = rt_ctx
                .discord_config
                .client
                .channel(channel_id)
                .exec()
                .await?
                .model()
                .await?;

            let gc = match channel {
                twilight_model::channel::Channel::Guild(gc) => gc,
                _ => return Err(anyhow::anyhow!("Unknown channel")),
            };

            if matches!(gc.guild_id(), Some(guild_id) if guild_id == rt_ctx.guild_id) {
                Ok(gc)
            } else {
                Err(anyhow::anyhow!("Unknown channel"))
            }
        }
    }
}

pub(crate) fn parse_str_snowflake_id(id_str: &str) -> Result<Id<GenericMarker>, AnyError> {
    if let Some(id) = Id::new_checked(id_str.parse()?) {
        Ok(id)
    } else {
        Err(anyhow::anyhow!("invalid channel id"))
    }
}
