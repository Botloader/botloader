use anyhow::anyhow;
use deno_core::{op_async, op_sync, Extension, OpState};
use futures::TryFutureExt;
use runtime_models::discord::guild::Ban;
use runtime_models::discord::message::{MessageFlags, SendEmoji};
use runtime_models::discord::user::User;
use runtime_models::discord::util::AuditLogExtras;
use runtime_models::internal::interactions::InteractionCallback;
use runtime_models::internal::member::UpdateGuildMemberFields;
use runtime_models::internal::misc_op::{CreateBanFields, GetReactionsFields};
use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};
use twilight_http::request::AuditLogReason;
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_model::id::marker::{MessageMarker, RoleMarker};
use twilight_model::id::Id;
use vm::{AnyError, JsValue};

use super::{get_guild_channel, parse_get_guild_channel, parse_str_snowflake_id};
use crate::dummy_op;
use crate::RuntimeContext;
use runtime_models::{
    discord::{guild::Guild, message::Message},
    internal::messages::{
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
            ("discord_edit_message", op_async(op_edit_channel_message)),
            ("discord_delete_message", op_async(op_delete_message)),
            (
                "discord_bulk_delete_messages",
                op_async(op_delete_messages_bulk),
            ),
            // Reactions
            ("discord_create_reaction", op_async(discord_create_reaction)),
            (
                "discord_delete_own_reaction",
                op_async(discord_delete_own_reaction),
            ),
            (
                "discord_delete_user_reaction",
                op_async(discord_delete_user_reaction),
            ),
            ("discord_get_reactions", op_async(discord_get_reactions)),
            (
                "discord_delete_all_reactions",
                op_async(discord_delete_all_reactions),
            ),
            (
                "discord_delete_all_reactions_for_emoji",
                op_async(discord_delete_all_reactions_for_emoji),
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
            ("discord_remove_member", op_async(op_remove_member)),
            ("discord_get_members", op_async(op_get_members)),
            ("discord_update_member", op_async(discord_update_member)),
            ("discord_add_member_role", op_async(discord_add_member_role)),
            (
                "discord_remove_member_role",
                op_async(discord_remove_member_role),
            ),
            // interactions
            (
                "discord_interaction_callback",
                op_async(op_interaction_callback),
            ),
            (
                "discord_interaction_followup",
                op_async(op_create_followup_message),
            ),
            (
                "discord_interaction_delete_original",
                op_async(op_interaction_delete_original),
            ),
            (
                "discord_interaction_delete_followup",
                op_async(op_interaction_delete_followup),
            ),
            // Bans
            ("discord_create_ban", op_async(op_create_ban)),
            ("discord_get_ban", op_async(op_get_ban)),
            ("discord_get_bans", op_async(op_get_bans)),
            ("discord_delete_ban", op_async(op_remove_ban)),
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

// Messages
pub async fn op_get_message(
    state: Rc<RefCell<OpState>>,
    args: OpGetMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = parse_get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_id = if let Some(id) = Id::new_checked(args.message_id.parse()?) {
        id
    } else {
        return Err(anyhow::anyhow!("invalid message id"));
    };

    let message = rt_ctx
        .discord_config
        .client
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

    let channel = parse_get_guild_channel(&rt_ctx, &args.channel_id).await?;

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
        .discord_config
        .client
        .channel_messages(channel.id())
        .limit(limit as u64)
        .unwrap();

    let res = if let Some(before) = args.before {
        let message_id = if let Some(id) = Id::new_checked(before.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid message id"));
        };

        req.before(message_id).exec().await
    } else if let Some(after) = args.after {
        let message_id = if let Some(id) = Id::new_checked(after.parse()?) {
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

    let channel = parse_get_guild_channel(&rt_ctx, &args.channel_id).await?;

    let maybe_embeds = args
        .fields
        .embeds
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    let components = args
        .fields
        .components
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    let mut mc = rt_ctx
        .discord_config
        .client
        .create_message(channel.id())
        .embeds(&maybe_embeds)?
        .components(&components)?;

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

    let channel = parse_get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_id = parse_str_snowflake_id(&args.message_id)?;

    let maybe_embeds = args
        .fields
        .embeds
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let components = args
        .fields
        .components
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let mut mc = rt_ctx
        .discord_config
        .client
        .update_message(channel.id(), message_id.cast())
        .content(args.fields.content.as_deref())?
        .components(components.as_deref())?;

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

    let components = args
        .fields
        .components
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    let interaction_client = rt_ctx.discord_config.interaction_client();

    let mut mc = interaction_client
        .create_followup_message(&args.interaction_token)
        .embeds(&maybe_embeds)?
        .components(&components)?;

    if matches!(
        args.flags,
        Some(MessageFlags {
            ephemeral: Some(true),
            ..
        })
    ) {
        mc = mc.ephemeral(true);
    }

    if let Some(content) = &args.fields.content {
        mc = mc.content(content)?
    }

    let mentions = args.fields.allowed_mentions.map(Into::into);
    if let Some(mentions) = &mentions {
        mc = mc.allowed_mentions(mentions);
    }

    Ok(mc.exec().await?.model().await?.into())
}

pub async fn op_interaction_callback(
    state: Rc<RefCell<OpState>>,
    args: InteractionCallback,
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let client = rt_ctx.discord_config.interaction_client();
    client
        .interaction_callback(
            Id::from_str(&args.interaction_id)?,
            &args.ineraction_token,
            &args.data.into(),
        )
        .exec()
        .await?;
    Ok(())
}

pub async fn op_interaction_delete_original(
    state: Rc<RefCell<OpState>>,
    token: String,
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let client = rt_ctx.discord_config.interaction_client();
    client.delete_interaction_original(&token).exec().await?;
    Ok(())
}

pub async fn op_interaction_delete_followup(
    state: Rc<RefCell<OpState>>,
    token: String,
    id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let client = rt_ctx.discord_config.interaction_client();
    client.delete_followup_message(&token, id).exec().await?;
    Ok(())
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

    let channel = parse_get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_id = parse_str_snowflake_id(&args.message_id)?;

    rt_ctx
        .discord_config
        .client
        .delete_message(channel.id(), message_id.cast())
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

    let channel = parse_get_guild_channel(&rt_ctx, &args.channel_id).await?;
    let message_ids = args
        .message_ids
        .iter()
        .filter_map(|v| parse_str_snowflake_id(v).ok())
        .map(|v| v.cast())
        .collect::<Vec<_>>();

    rt_ctx
        .discord_config
        .client
        .delete_messages(channel.id(), &message_ids)
        .exec()
        .await?;

    Ok(())
}

// Roles
pub async fn op_get_role(
    state: Rc<RefCell<OpState>>,
    role_id: Id<RoleMarker>,
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

// Reactions
pub async fn discord_create_reaction(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .create_reaction(channel_id, message_id, &(&emoji).into())
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_delete_own_reaction(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_current_user_reaction(channel_id, message_id, &(&emoji).into())
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_delete_user_reaction(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id, user_id): (Id<ChannelMarker>, Id<MessageMarker>, Id<UserMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_reaction(channel_id, message_id, &(&emoji).into(), user_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_get_reactions(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    fields: GetReactionsFields,
) -> Result<Vec<User>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let _ = get_guild_channel(&rt_ctx, channel_id).await?;

    let emoji = (&fields.emoji).into();

    let mut req = rt_ctx
        .discord_config
        .client
        .reactions(channel_id, message_id, &emoji);

    if let Some(after_str) = &fields.after {
        req = req.after(parse_str_snowflake_id(after_str)?.cast())
    }
    if let Some(limit) = fields.limit {
        req = req.limit(limit.into())?;
    }

    Ok(req
        .exec()
        .await?
        .model()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn discord_delete_all_reactions(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let _ = get_guild_channel(&rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_all_reactions(channel_id, message_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_delete_all_reactions_for_emoji(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let _ = get_guild_channel(&rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_all_reaction(channel_id, message_id, &(&emoji).into())
        .exec()
        .await?;

    Ok(())
}

// Channels
pub async fn op_get_channel(
    state: Rc<RefCell<OpState>>,
    channel_id_str: String,
    _: (),
) -> Result<runtime_models::discord::channel::GuildChannel, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let channel = parse_get_guild_channel(&rt_ctx, &channel_id_str).await?;
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

// Members
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
        .map(|v| v.parse().map(Id::new_checked).ok().flatten())
        .collect::<Vec<_>>();

    let mut res = Vec::new();
    for item in ids {
        if let Some(id) = item {
            // fall back to http api
            let member = rt_ctx
                .discord_config
                .client
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
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    rt_ctx
        .discord_config
        .client
        .add_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_remove_member_role(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    rt_ctx
        .discord_config
        .client
        .remove_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn discord_update_member(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    fields: UpdateGuildMemberFields,
) -> Result<runtime_models::discord::member::Member, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };
    let mut builder = rt_ctx
        .discord_config
        .client
        .update_guild_member(rt_ctx.guild_id, user_id);

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

// Bans
pub async fn op_create_ban(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    extras: CreateBanFields,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let mut req = rt_ctx
        .discord_config
        .client
        .create_ban(rt_ctx.guild_id, user_id);

    if let Some(days) = extras.delete_message_days {
        req = req.delete_message_days(days.into())?;
    }

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.exec().await?;

    Ok(())
}

pub async fn op_get_ban(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    _: (),
) -> Result<Ban, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let result = rt_ctx
        .discord_config
        .client
        .ban(rt_ctx.guild_id, user_id)
        .exec()
        .await?
        .model()
        .await?;

    Ok(result.into())
}

pub async fn op_get_bans(state: Rc<RefCell<OpState>>, _: (), _: ()) -> Result<Vec<Ban>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let result = rt_ctx
        .discord_config
        .client
        .bans(rt_ctx.guild_id)
        .exec()
        .await?
        .model()
        .await?;

    Ok(result.into_iter().map(Into::into).collect())
}

pub async fn op_remove_ban(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    extras: AuditLogExtras,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let mut req = rt_ctx
        .discord_config
        .client
        .delete_ban(rt_ctx.guild_id, user_id);

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.exec().await?;

    Ok(())
}

// Other
pub async fn op_remove_member(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    extras: AuditLogExtras,
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let mut req = rt_ctx
        .discord_config
        .client
        .remove_guild_member(rt_ctx.guild_id, user_id);

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.exec().await?;

    Ok(())
}
