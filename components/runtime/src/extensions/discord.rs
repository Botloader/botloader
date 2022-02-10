use anyhow::anyhow;
use deno_core::OpState;
use deno_core::{op_async, op_sync, Extension};
use futures::TryFutureExt;
use runtime_models::ops::member::UpdateGuildMemberFields;
use std::{cell::RefCell, rc::Rc};
use twilight_model::id::UserId;
use twilight_model::id::{MessageId, RoleId};
use vm::{AnyError, JsValue};

use super::{get_guild_channel, parse_str_snowflake_id};
use crate::dummy_op;
use crate::RuntimeContext;
use runtime_models::{
    discord::{guild::Guild, message::Message},
    ops::messages::{
        OpCreateChannelMessage, OpCreateFollowUpMessage, OpDeleteMessage, OpDeleteMessagesBulk,
        OpEditChannelMessage, OpGetMessage, OpGetMessages,
    },
};

pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![
            // guild
            ("discord_get_guild", op_async(op_get_guild)),
            ("discord_edit_guild", op_sync(dummy_op)),
            // messages
            ("discord_get_message", op_async(op_get_message)),
            ("discord_get_messages", op_async(op_get_messages)),
            (
                "discord_create_message",
                op_async(op_create_channel_message),
            ),
            (
                "discord_create_followup_message",
                op_async(op_create_followup_message),
            ),
            ("discord_edit_message", op_async(op_edit_channel_message)),
            ("discord_delete_message", op_async(op_delete_message)),
            (
                "discord_bulk_delete_messages",
                op_async(op_delete_messages_bulk),
            ),
            // roles
            ("discord_get_role", op_async(op_get_role)),
            ("discord_get_roles", op_async(op_get_roles)),
            ("discord_create_role", op_sync(dummy_op)),
            ("discord_edit_role", op_sync(dummy_op)),
            ("discord_delete_role", op_sync(dummy_op)),
            // channels
            ("discord_get_channel", op_async(op_get_channel)),
            ("discord_get_channels", op_async(op_get_channels)),
            ("discord_create_channel", op_sync(dummy_op)),
            ("discord_edit_channel", op_sync(dummy_op)),
            ("discord_delete_channel", op_sync(dummy_op)),
            // invites
            ("discord_get_invite", op_sync(dummy_op)),
            ("discord_get_invites", op_sync(dummy_op)),
            ("discord_create_invite", op_sync(dummy_op)),
            ("discord_delete_invite", op_sync(dummy_op)),
            // members
            ("discord_get_members", op_async(op_get_members)),
            ("discord_update_member", op_async(discord_update_member)),
            ("discord_add_member_role", op_async(discord_add_member_role)),
            (
                "discord_remove_member_role",
                op_async(discord_remove_member_role),
            ),
        ])
        .build()
}

pub async fn op_get_guild(
    state: Rc<RefCell<OpState>>,
    _args: JsValue,
    _: (),
) -> Result<Guild, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    match rt_ctx
        .bot_state
        .get_guild(rt_ctx.guild_id)
        .map_err(|err| anyhow::anyhow!("error calling state api: {}", err))
        .await?
    {
        Some(c) => Ok(c.into()),
        None => Err(anyhow::anyhow!("guild not in state")),
    }
}

pub async fn op_get_message(
    state: Rc<RefCell<OpState>>,
    args: OpGetMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_id = if let Some(id) = MessageId::new(args.message_id.parse()?) {
        id
    } else {
        return Err(anyhow::anyhow!("invalid message id"));
    };

    let message = rt_ctx
        .dapi
        .message(channel.id(), message_id)
        .exec()
        .await?
        .model()
        .await?;

    Ok(message.into())
}

pub async fn op_get_messages(
    state: Rc<RefCell<OpState>>,
    args: OpGetMessages,
    _: (),
) -> Result<Vec<Message>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &args.channel_id).await?;

    let limit = if let Some(limit) = args.limit {
        if limit > 100 {
            100
        } else if limit < 1 {
            1
        } else {
            limit
        }
    } else {
        50
    };

    let req = rt_ctx
        .dapi
        .channel_messages(channel.id())
        .limit(limit as u64)
        .unwrap();

    let res = if let Some(before) = args.before {
        let message_id = if let Some(id) = MessageId::new(before.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid message id"));
        };

        req.before(message_id).exec().await
    } else if let Some(after) = args.after {
        let message_id = if let Some(id) = MessageId::new(after.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid message id"));
        };

        req.after(message_id).exec().await
    } else {
        req.exec().await
    };

    let messages = res?.model().await?;
    Ok(messages.into_iter().map(Into::into).collect())
}

pub async fn op_create_channel_message(
    state: Rc<RefCell<OpState>>,
    args: OpCreateChannelMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &args.channel_id).await?;

    let maybe_embeds = args
        .fields
        .embeds
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    let mut mc = rt_ctx
        .dapi
        .create_message(channel.id())
        .embeds(&maybe_embeds)?;

    if let Some(content) = &args.fields.content {
        mc = mc.content(content)?
    }

    if let Some(mentions) = args.fields.allowed_mentions {
        mc = mc.allowed_mentions(mentions.into());
    }

    Ok(mc.exec().await?.model().await?.into())
}

pub async fn op_edit_channel_message(
    state: Rc<RefCell<OpState>>,
    args: OpEditChannelMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_id = parse_str_snowflake_id(&args.message_id)?;

    let maybe_embeds = args
        .fields
        .embeds
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let mut mc = rt_ctx
        .dapi
        .update_message(channel.id(), message_id.0.into())
        .content(args.fields.content.as_deref())?;

    if let Some(embeds) = &maybe_embeds {
        mc = mc.embeds(embeds)?;
    }

    if let Some(mentions) = args.fields.allowed_mentions {
        mc = mc.allowed_mentions(mentions.into());
    }

    Ok(mc.exec().await?.model().await?.into())
}

pub async fn op_create_followup_message(
    state: Rc<RefCell<OpState>>,
    args: OpCreateFollowUpMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let maybe_embeds = args
        .fields
        .embeds
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    let mut mc = rt_ctx
        .dapi
        .create_followup_message(&args.interaction_token)
        .unwrap()
        .embeds(&maybe_embeds);

    if let Some(content) = &args.fields.content {
        mc = mc.content(content)
    }

    Ok(mc.exec().await?.model().await?.into())
}

pub async fn op_delete_message(
    state: Rc<RefCell<OpState>>,
    args: OpDeleteMessage,
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_id = parse_str_snowflake_id(&args.message_id)?;

    rt_ctx
        .dapi
        .delete_message(channel.id(), message_id.0.into())
        .exec()
        .await?;

    Ok(())
}

pub async fn op_delete_messages_bulk(
    state: Rc<RefCell<OpState>>,
    args: OpDeleteMessagesBulk,
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_ids = args
        .message_ids
        .iter()
        .filter_map(|v| parse_str_snowflake_id(v).ok())
        .map(|v| v.0.into())
        .collect::<Vec<_>>();

    rt_ctx
        .dapi
        .delete_messages(channel.id(), &message_ids)
        .exec()
        .await?;

    Ok(())
}

pub async fn op_get_role(
    state: Rc<RefCell<OpState>>,
    role_id: RoleId,
    _: (),
) -> Result<runtime_models::discord::role::Role, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    match rt_ctx.bot_state.get_role(rt_ctx.guild_id, role_id).await? {
        Some(c) => Ok(c.into()),
        _ => Err(anyhow::anyhow!("role not in state")),
    }
}

pub async fn op_get_roles(
    state: Rc<RefCell<OpState>>,
    _: (),
    _: (),
) -> Result<Vec<runtime_models::discord::role::Role>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let roles = rt_ctx.bot_state.get_roles(rt_ctx.guild_id).await?;
    Ok(roles.into_iter().map(Into::into).collect())
}

pub async fn op_get_channel(
    state: Rc<RefCell<OpState>>,
    channel_id_str: String,
    _: (),
) -> Result<runtime_models::discord::channel::GuildChannel, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = get_guild_channel(&rt_ctx, &channel_id_str).await?;
    Ok(channel.into())
}

pub async fn op_get_channels(
    state: Rc<RefCell<OpState>>,
    _: (),
    _: (),
) -> Result<Vec<runtime_models::discord::channel::GuildChannel>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channels = rt_ctx.bot_state.get_channels(rt_ctx.guild_id).await?;
    Ok(channels.into_iter().map(Into::into).collect())
}

pub async fn op_get_members(
    state: Rc<RefCell<OpState>>,
    user_ids: Vec<String>,
    _: (),
) -> Result<Vec<Option<runtime_models::discord::member::Member>>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    if user_ids.len() > 100 {
        return Err(anyhow!("too many user ids provided, max 100"));
    }

    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    let ids = user_ids
        .into_iter()
        .map(|v| v.parse().map(UserId::new).ok().flatten())
        .collect::<Vec<_>>();

    let mut res = Vec::new();
    for item in ids {
        if let Some(id) = item {
            // fall back to http api
            let member = rt_ctx
                .dapi
                .guild_member(rt_ctx.guild_id, id)
                .exec()
                .await?
                .model()
                .await?;

            res.push(Some(member.into()))
        } else {
            res.push(None)
        }
    }

    Ok(res)
}

pub async fn discord_add_member_role(
    state: Rc<RefCell<OpState>>,
    user_id: UserId,
    role_id: RoleId,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    rt_ctx
        .dapi
        .add_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_remove_member_role(
    state: Rc<RefCell<OpState>>,
    user_id: UserId,
    role_id: RoleId,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    rt_ctx
        .dapi
        .remove_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_update_member(
    state: Rc<RefCell<OpState>>,
    user_id: UserId,
    fields: UpdateGuildMemberFields,
) -> Result<runtime_models::discord::member::Member, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };
    let mut builder = rt_ctx.dapi.update_guild_member(rt_ctx.guild_id, user_id);

    if let Some(maybe_cid) = fields.channel_id {
        builder = builder.channel_id(maybe_cid);
    }

    if let Some(deaf) = fields.deaf {
        builder = builder.deaf(deaf);
    }

    if let Some(mute) = fields.mute {
        builder = builder.mute(mute);
    }

    if let Some(maybe_nick) = &fields.nick {
        builder = builder.nick(maybe_nick.as_deref())?
    }

    if let Some(roles) = &fields.roles {
        builder = builder.roles(roles);
    }

    let ret = builder.exec().await?.model().await?;
    Ok(ret.into())
}
