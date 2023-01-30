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

use self::discord::{handle_discord_error, not_found_error};

pub mod console;
pub mod discord;
pub mod httpclient;
pub mod storage;
pub mod tasks;

// ensures the provided channel is in the guild, also checking the api as fallback
pub(crate) async fn parse_get_guild_channel(
    state: &Rc<RefCell<OpState>>,
    rt_ctx: &RuntimeContext,
    channel_id_str: &str,
) -> Result<Channel, AnyError> {
    let channel_id = if let Some(channel_id) = Id::new_checked(channel_id_str.parse()?) {
        channel_id
    } else {
        return Err(anyhow::anyhow!("invalid channel id"));
    };

    get_guild_channel(state, rt_ctx, channel_id).await
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
            let channel = rt_ctx
                .discord_config
                .client
                .channel(channel_id)
                .await
                .map_err(|err| handle_discord_error(state, err))?
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
