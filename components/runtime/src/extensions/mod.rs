use std::{cell::RefCell, rc::Rc};

use deno_core::OpState;
use twilight_model::{
    channel::Channel,
    id::{
        marker::{ChannelMarker, GenericMarker},
        Id,
    },
};
use vm::AnyError;

use crate::RuntimeContext;

use self::discord::{discord_request, not_found_error};

pub mod console;
pub mod discord;
pub mod httpclient;
pub mod storage;
pub mod tasks;

pub(crate) fn parse_discord_id<T>(input: &str) -> Result<Id<T>, AnyError> {
    if let Some(id) = Id::new_checked(input.parse()?) {
        Ok(id)
    } else {
        Err(anyhow::anyhow!("invalid discord snowflake"))
    }
}

// ensures the provided channel is in the guild, also checking the api as fallback
pub(crate) async fn parse_get_guild_channel(
    state: &Rc<RefCell<OpState>>,
    rt_ctx: &RuntimeContext,
    channel_id_str: &str,
) -> Result<Channel, AnyError> {
    get_guild_channel(state, rt_ctx, parse_discord_id(channel_id_str)?).await
}

pub(crate) async fn get_guild_channel(
    state: &Rc<RefCell<OpState>>,
    rt_ctx: &RuntimeContext,
    channel_id: Id<ChannelMarker>,
) -> Result<Channel, AnyError> {
    match rt_ctx
        .bot_state
        .get_channel(rt_ctx.guild_id, channel_id)
        .await?
    {
        Some(c) => {
            if !matches!(c.guild_id, Some(guild_id) if guild_id == rt_ctx.guild_id) {
                Err(not_found_error(format!("channel `{channel_id} not found`")))
            } else {
                Ok(c)
            }
        }
        None => {
            let cloned_discord = rt_ctx.discord_config.clone();

            let channel = discord_request(state, async move {
                cloned_discord.client.channel(channel_id).await
            })
            .await?
            .model()
            .await?;

            if matches!(channel.guild_id, Some(guild_id) if guild_id == rt_ctx.guild_id) {
                Ok(channel)
            } else {
                Err(not_found_error(format!("channel `{channel_id} not found`")))
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
