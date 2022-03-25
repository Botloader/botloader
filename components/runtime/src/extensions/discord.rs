use anyhow::anyhow;
use deno_core::{error::custom_error, op, Extension, OpState};
use futures::TryFutureExt;
use runtime_models::{
    discord::{
        guild::Guild,
        message::{MessageFlags, SendEmoji},
        util::AuditLogExtras,
    },
    internal::{
        interactions::InteractionCallback,
        member::{Ban, UpdateGuildMemberFields},
        messages::{
            Message, OpCreateChannelMessage, OpCreateFollowUpMessage, OpDeleteMessage,
            OpDeleteMessagesBulk, OpEditChannelMessage, OpGetMessages,
        },
        misc_op::{CreateBanFields, GetReactionsFields},
        user::User,
    },
};
use std::{borrow::Cow, str::FromStr};
use std::{cell::RefCell, rc::Rc};
use twilight_http::error::ErrorType;
use twilight_http::request::AuditLogReason;
use twilight_http::{
    api_error::{ApiError, GeneralApiError},
    response::StatusCode,
};
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_model::id::marker::{MessageMarker, RoleMarker};
use twilight_model::id::Id;
use vm::AnyError;

use super::{get_guild_channel, parse_get_guild_channel, parse_str_snowflake_id};
use crate::get_rt_ctx;

pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![
            // guild
            op_discord_get_guild::decl(),
            // messages
            op_discord_get_message::decl(),
            op_discord_get_messages::decl(),
            op_discord_create_message::decl(),
            op_discord_edit_message::decl(),
            op_discord_delete_message::decl(),
            op_discord_bulk_delete_messages::decl(),
            // Reactions
            op_discord_create_reaction::decl(),
            op_discord_delete_own_reaction::decl(),
            op_discord_delete_user_reaction::decl(),
            op_discord_get_reactions::decl(),
            op_discord_delete_all_reactions::decl(),
            op_discord_delete_all_reactions_for_emoji::decl(),
            // roles
            op_discord_get_role::decl(),
            op_discord_get_roles::decl(),
            // channels
            op_discord_get_channel::decl(),
            op_discord_get_channels::decl(),
            // invites
            // members
            op_discord_remove_member::decl(),
            op_discord_get_members::decl(),
            op_discord_update_member::decl(),
            op_discord_add_member_role::decl(),
            op_discord_remove_member_role::decl(),
            // interactions
            op_discord_interaction_callback::decl(),
            op_discord_interaction_get_original_response::decl(),
            op_discord_interaction_edit_original_response::decl(),
            op_discord_interaction_delete_original::decl(),
            op_discord_interaction_followup_message::decl(),
            op_discord_interaction_get_followup_message::decl(),
            op_discord_interaction_edit_followup_message::decl(),
            op_discord_interaction_delete_followup_message::decl(),
            // Bans
            op_discord_create_ban::decl(),
            op_discord_get_ban::decl(),
            op_discord_get_bans::decl(),
            op_discord_delete_ban::decl(),
        ])
        .build()
}

pub fn handle_discord_error(_state: &Rc<RefCell<OpState>>, err: twilight_http::Error) -> AnyError {
    let kind = err.kind();
    match kind {
        ErrorType::Response {
            // 10008 is unknown message
            error: ApiError::General(GeneralApiError { code, message, .. }),
            status,
            ..
        } => error_from_code(*status, *code, message),
        _ => err.into(),
    }
}

pub fn error_from_code(resp_code: StatusCode, code: u64, message: &str) -> AnyError {
    match resp_code.raw() {
        404 => not_found_error(format!("{code}: {message}")),
        403 => custom_error("DiscordPermissionsError", format!("{code}: {message}")),
        400..=499 => match code {
            30001..=40000 => custom_error("DiscordLimitReachedError", format!("{code}: {message}")),
            _ => custom_error("DiscordGenericErrorResponse", format!("{code}: {message}")),
        },
        status @ 500..=599 => custom_error(
            "DiscordServerErrorResponse",
            format!(
                "Discord failed handling the request (5xx response), http status: {status}, code: \
                 {code}, message: {message}"
            ),
        ),
        other => custom_error(
            "DiscordGenericErrorResponse",
            format!(
                "An error occured with the discord API, http status: {other}, code: {code}, \
                 message: {message}"
            ),
        ),
    }
}

pub fn not_found_error(message: impl Into<Cow<'static, str>>) -> AnyError {
    custom_error("DiscordNotFoundError", message)
}

#[op]
pub async fn op_discord_get_guild(state: Rc<RefCell<OpState>>) -> Result<Guild, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

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
#[op]
pub async fn op_discord_get_message(
    state: Rc<RefCell<OpState>>,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    Ok(rt_ctx
        .discord_config
        .client
        .message(channel.id(), message_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_get_messages(
    state: Rc<RefCell<OpState>>,
    args: OpGetMessages,
) -> Result<Vec<Message>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;

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
        .limit(limit as u64)?;

    let res = if let Some(before) = args.before {
        let message_id = if let Some(id) = Id::new_checked(before.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid 'before' message id"));
        };

        req.before(message_id).exec().await
    } else if let Some(after) = args.after {
        let message_id = if let Some(id) = Id::new_checked(after.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid 'after' message id"));
        };

        req.after(message_id).exec().await
    } else {
        req.exec().await
    };

    let messages = res
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(messages.into_iter().map(Into::into).collect())
}

#[op]
pub async fn op_discord_create_message(
    state: Rc<RefCell<OpState>>,
    args: OpCreateChannelMessage,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;

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

    Ok(mc
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_edit_message(
    state: Rc<RefCell<OpState>>,
    args: OpEditChannelMessage,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;
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

    Ok(mc
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_interaction_callback(
    state: Rc<RefCell<OpState>>,
    args: InteractionCallback,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    client
        .interaction_callback(
            Id::from_str(&args.interaction_id)?,
            &args.ineraction_token,
            &args.data.into(),
        )
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_interaction_get_original_response(
    state: Rc<RefCell<OpState>>,
    token: String,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    Ok(client
        .get_interaction_original(&token)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_interaction_edit_original_response(
    state: Rc<RefCell<OpState>>,
    args: OpCreateFollowUpMessage,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let maybe_embeds = args
        .fields
        .embeds
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let components = args
        .fields
        .components
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let interaction_client = rt_ctx.discord_config.interaction_client();

    let mut mc = interaction_client
        .update_interaction_original(&args.interaction_token)
        .content(args.fields.content.as_deref())?
        .embeds(maybe_embeds.as_deref())?
        .components(components.as_deref())?
        .content(args.fields.content.as_deref())?;

    if let Some(mentions) = args.fields.allowed_mentions {
        mc = mc.allowed_mentions(mentions.into());
    }

    Ok(mc
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_interaction_delete_original(
    state: Rc<RefCell<OpState>>,
    token: String,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    client.delete_interaction_original(&token).exec().await?;
    Ok(())
}

#[op]
pub async fn op_discord_interaction_get_followup_message(
    state: Rc<RefCell<OpState>>,
    token: String,
    id: Id<MessageMarker>,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    Ok(client
        .followup_message(&token, id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_interaction_followup_message(
    state: Rc<RefCell<OpState>>,
    args: OpCreateFollowUpMessage,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

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

    Ok(mc
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op]
pub async fn op_discord_interaction_edit_followup_message(
    state: Rc<RefCell<OpState>>,
    message_id: Id<MessageMarker>,
    args: OpCreateFollowUpMessage,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let maybe_embeds = args
        .fields
        .embeds
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let components = args
        .fields
        .components
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let interaction_client = rt_ctx.discord_config.interaction_client();

    let mut mc = interaction_client
        .update_followup_message(&args.interaction_token, message_id)
        .content(args.fields.content.as_deref())?
        .embeds(maybe_embeds.as_deref())?
        .components(components.as_deref())?
        .content(args.fields.content.as_deref())?;

    if let Some(mentions) = args.fields.allowed_mentions {
        mc = mc.allowed_mentions(mentions.into());
    }

    mc.exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_interaction_delete_followup_message(
    state: Rc<RefCell<OpState>>,
    token: String,
    id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    client
        .delete_followup_message(&token, id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;
    Ok(())
}

#[op]
pub async fn op_discord_delete_message(
    state: Rc<RefCell<OpState>>,
    args: OpDeleteMessage,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;
    let message_id = parse_str_snowflake_id(&args.message_id)?;

    rt_ctx
        .discord_config
        .client
        .delete_message(channel.id(), message_id.cast())
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_bulk_delete_messages(
    state: Rc<RefCell<OpState>>,
    args: OpDeleteMessagesBulk,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;
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
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Roles
#[op]
pub async fn op_discord_get_role(
    state: Rc<RefCell<OpState>>,
    role_id: Id<RoleMarker>,
) -> Result<runtime_models::discord::role::Role, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    match rt_ctx.bot_state.get_role(rt_ctx.guild_id, role_id).await? {
        Some(c) => Ok(c.into()),
        _ => Err(not_found_error(format!("role `{role_id}` not found"))),
    }
}

#[op]
pub async fn op_discord_get_roles(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::discord::role::Role>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let roles = rt_ctx.bot_state.get_roles(rt_ctx.guild_id).await?;
    Ok(roles.into_iter().map(Into::into).collect())
}

// Reactions
#[op]
pub async fn op_discord_create_reaction(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .create_reaction(channel_id, message_id, &(&emoji).into())
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_delete_own_reaction(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_current_user_reaction(channel_id, message_id, &(&emoji).into())
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_delete_user_reaction(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id, user_id): (Id<ChannelMarker>, Id<MessageMarker>, Id<UserMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_reaction(channel_id, message_id, &(&emoji).into(), user_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_get_reactions(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    fields: GetReactionsFields,
) -> Result<Vec<User>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

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
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[op]
pub async fn op_discord_delete_all_reactions(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_all_reactions(channel_id, message_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_delete_all_reactions_for_emoji(
    state: Rc<RefCell<OpState>>,
    (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_all_reaction(channel_id, message_id, &(&emoji).into())
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Channels
#[op]
pub async fn op_discord_get_channel(
    state: Rc<RefCell<OpState>>,
    channel_id_str: String,
) -> Result<runtime_models::internal::channel::GuildChannel, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &channel_id_str).await?;
    Ok(channel.into())
}

#[op]
pub async fn op_discord_get_channels(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::internal::channel::GuildChannel>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channels = rt_ctx.bot_state.get_channels(rt_ctx.guild_id).await?;
    Ok(channels.into_iter().map(Into::into).collect())
}

// Members
#[op]
pub async fn op_discord_get_members(
    state: Rc<RefCell<OpState>>,
    user_ids: Vec<String>,
) -> Result<Vec<Option<runtime_models::internal::member::Member>>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

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
            match rt_ctx
                .discord_config
                .client
                .guild_member(rt_ctx.guild_id, id)
                .exec()
                .await
            {
                Ok(next) => {
                    let member = next.model().await?;
                    res.push(Some(member.into()))
                }
                Err(err) => {
                    if !matches!(
                        err.kind(),
                        ErrorType::Response {
                            // 10007 is unknown member
                            error: ApiError::General(GeneralApiError { code: 10007, .. }),
                            ..
                        },
                    ) {
                        return Err(handle_discord_error(&state, err));
                    }

                    res.push(None);
                }
            }
        } else {
            res.push(None)
        }
    }

    Ok(res)
}

#[op]
pub async fn op_discord_add_member_role(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    rt_ctx
        .discord_config
        .client
        .add_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_remove_member_role(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    rt_ctx
        .discord_config
        .client
        .remove_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_update_member(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    fields: UpdateGuildMemberFields,
) -> Result<runtime_models::internal::member::Member, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
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

    let ret = builder
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;
    Ok(ret.into())
}

// Bans
#[op]
pub async fn op_discord_create_ban(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    extras: CreateBanFields,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

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

    req.exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op]
pub async fn op_discord_get_ban(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
) -> Result<Ban, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let result = rt_ctx
        .discord_config
        .client
        .ban(rt_ctx.guild_id, user_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(result.into())
}

#[op]
pub async fn op_discord_get_bans(state: Rc<RefCell<OpState>>) -> Result<Vec<Ban>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let result = rt_ctx
        .discord_config
        .client
        .bans(rt_ctx.guild_id)
        .exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(result.into_iter().map(Into::into).collect())
}

#[op]
pub async fn op_discord_delete_ban(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    extras: AuditLogExtras,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let mut req = rt_ctx
        .discord_config
        .client
        .delete_ban(rt_ctx.guild_id, user_id);

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Other
#[op]
pub async fn op_discord_remove_member(
    state: Rc<RefCell<OpState>>,
    user_id: Id<UserMarker>,
    extras: AuditLogExtras,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let mut req = rt_ctx
        .discord_config
        .client
        .remove_guild_member(rt_ctx.guild_id, user_id);

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.exec()
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}
