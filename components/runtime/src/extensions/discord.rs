use anyhow::anyhow;
use deno_core::{error::custom_error, op2, OpState};
use futures::TryFutureExt;
use runtime_models::{
    discord::{
        channel::{PermissionOverwrite, PermissionOverwriteType},
        guild::Guild,
        message::SendEmoji,
        util::AuditLogExtras,
    },
    internal::{
        channel::{CreateChannel, EditChannel},
        events::VoiceState,
        interactions::InteractionCallback,
        invite::CreateInviteFields,
        member::{Ban, UpdateGuildMemberFields},
        messages::{
            Message, OpCreateChannelMessage, OpCreateFollowUpMessage, OpDeleteMessage,
            OpDeleteMessagesBulk, OpEditChannelMessage, OpGetMessages,
        },
        misc_op::{CreateBanFields, GetReactionsFields},
        user::User,
    },
};
use std::{
    borrow::Cow,
    collections::VecDeque,
    str::FromStr,
    time::{Duration, Instant},
};
use std::{cell::RefCell, rc::Rc};
use tracing::{info, warn};
use twilight_http::error::ErrorType;
use twilight_http::request::AuditLogReason;
use twilight_http::{
    api_error::{ApiError, GeneralApiError},
    response::StatusCode,
};
use twilight_model::id::marker::{GenericMarker, MessageMarker, RoleMarker};
use twilight_model::id::Id;
use twilight_model::{
    guild::Permissions,
    id::marker::{ChannelMarker, UserMarker},
};
use vm::AnyError;

use super::{get_guild_channel, parse_get_guild_channel, parse_str_snowflake_id};
use crate::{get_rt_ctx, limits::RateLimiters, RuntimeContext, RuntimeEvent};

deno_core::extension!(
    bl_discord,
    ops = [
        // guild
        op_discord_get_guild,
        op_discord_get_invites,
        op_discord_get_invite,
        op_discord_delete_invite,
        // messages
        op_discord_get_message,
        op_discord_get_messages,
        op_discord_create_message,
        op_discord_edit_message,
        op_discord_crosspost_message,
        op_discord_delete_message,
        op_discord_bulk_delete_messages,
        // Reactions
        op_discord_create_reaction,
        op_discord_delete_own_reaction,
        op_discord_delete_user_reaction,
        op_discord_get_reactions,
        op_discord_delete_all_reactions,
        op_discord_delete_all_reactions_for_emoji,
        // roles
        op_discord_get_role,
        op_discord_get_roles,
        // channels
        op_discord_get_channel,
        op_discord_get_channels,
        op_discord_create_channel,
        op_discord_edit_channel,
        op_discord_delete_channel,
        op_discord_update_channel_permission,
        op_discord_delete_channel_permission,
        op_discord_get_channel_invites,
        op_discord_create_channel_invite,
        // voice
        op_discord_get_voice_states,
        // pins
        op_discord_get_channel_pins,
        op_discord_create_pin,
        op_discord_delete_pin,
        // invites
        // members
        op_discord_remove_member,
        op_discord_get_members,
        op_discord_update_member,
        op_discord_add_member_role,
        op_discord_remove_member_role,
        // interactions
        op_discord_interaction_callback,
        op_discord_interaction_get_original_response,
        op_discord_interaction_edit_original_response,
        op_discord_interaction_delete_original,
        op_discord_interaction_followup_message,
        op_discord_interaction_get_followup_message,
        op_discord_interaction_edit_followup_message,
        op_discord_interaction_delete_followup_message,
        // Bans
        op_discord_create_ban,
        op_discord_get_ban,
        op_discord_get_bans,
        op_discord_delete_ban,
        // misc
        op_discord_get_member_permissions,
    ],
    state = |state| {
        state.put(DiscordOpsState {
            recent_bad_requests: VecDeque::new(),
        });
        // state.put::<Options>(options.options);
    },
);

struct DiscordOpsState {
    recent_bad_requests: VecDeque<Instant>,
}

impl DiscordOpsState {
    fn add_failed_req(&mut self) {
        self.recent_bad_requests.push_back(Instant::now());

        while self.recent_bad_requests.len() > 29 {
            self.recent_bad_requests.pop_front();
        }
    }

    fn should_suspend_guild(&self) -> bool {
        if self.recent_bad_requests.len() < 29 {
            return false;
        }

        self.recent_bad_requests[0].elapsed() < Duration::from_secs(60)
    }
}

pub fn handle_discord_error(state: &Rc<RefCell<OpState>>, err: twilight_http::Error) -> AnyError {
    let kind = err.kind();
    if let ErrorType::Response { status, .. } = kind {
        // check if this guild has run into a lot of "invalid" requests
        //
        // this is needed because discord will ban our IP if we exceed 10_000 invalid req/10min as of writing
        let raw = status.get();
        if raw == 401 || raw == 403 || raw == 429 {
            let mut rc = state.borrow_mut();
            let dstate = rc.borrow_mut::<DiscordOpsState>();
            dstate.add_failed_req();
            if dstate.should_suspend_guild() {
                let handle = rc.borrow::<vm::vm::VmShutdownHandle>().clone();
                let rt_ctx = rc.borrow::<RuntimeContext>().clone();
                drop(rc);

                warn!(
                    guild_id = rt_ctx.guild_id.get(),
                    "guild hit >30 invalid requests within 60s, suspending it"
                );
                let _ = rt_ctx.event_tx.send(RuntimeEvent::InvalidRequestsExceeded);
                handle.shutdown_vm(vm::vm::ShutdownReason::Unknown, false);
            }
        }
    }

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
    match resp_code.get() {
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
                "An error occurred with the discord API, http status: {other}, code: {code}, \
                 message: {message}"
            ),
        ),
    }
}

pub fn not_found_error(message: impl Into<Cow<'static, str>>) -> AnyError {
    custom_error("DiscordNotFoundError", message)
}

#[op2(async)]
#[serde]
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

#[op2(async)]
#[serde]
pub async fn op_discord_get_invites(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::internal::invite::Invite>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let resp = rt_ctx
        .discord_config
        .client
        .guild_invites(rt_ctx.guild_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    resp.into_iter().map(TryInto::try_into).collect()
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_invite(
    state: Rc<RefCell<OpState>>,
    #[string] code: String,
    with_counts: bool,
    with_expiration: bool,
) -> Result<runtime_models::internal::invite::Invite, AnyError> {
    RateLimiters::discord_get_public_invite(&state).await;

    let rt_ctx = get_rt_ctx(&state);

    let mut req = rt_ctx.discord_config.client.invite(&code);
    if with_counts {
        req = req.with_counts();
    }

    if with_expiration {
        req = req.with_expiration();
    }

    req.await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .try_into()
}

#[op2(async)]
#[serde]
pub async fn op_discord_delete_invite(
    state: Rc<RefCell<OpState>>,
    #[string] code: String,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // we need to make sure this invite comes from this guild
    let invite = rt_ctx
        .discord_config
        .client
        .invite(&code)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    let is_correct_guild = if let Some(guild) = invite.guild {
        guild.id == rt_ctx.guild_id
    } else {
        false
    };

    // someone tried to be sneaky...
    if !is_correct_guild {
        return Err(anyhow!("This invite does not belong to your server."));
    }

    rt_ctx
        .discord_config
        .client
        .delete_invite(&code)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_voice_states(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<VoiceState>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let voice_states = rt_ctx
        .bot_state
        .get_guild_voice_states(rt_ctx.guild_id)
        .await?;

    Ok(voice_states
        .into_iter()
        .filter_map(|v| v.try_into().ok())
        .collect())
}

// Messages
#[op2(async)]
#[serde]
pub async fn op_discord_get_message(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] message_id: Id<MessageMarker>,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    Ok(rt_ctx
        .discord_config
        .client
        .message(channel.id, message_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_messages(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpGetMessages,
) -> Result<Vec<Message>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;

    let limit = if let Some(limit) = args.limit {
        limit.clamp(1, 100)
    } else {
        50
    };

    let req = rt_ctx
        .discord_config
        .client
        .channel_messages(channel.id)
        .limit(limit as u16)?;

    let res = if let Some(before) = args.before {
        let message_id = if let Some(id) = Id::new_checked(before.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid 'before' message id"));
        };

        req.before(message_id).await
    } else if let Some(after) = args.after {
        let message_id = if let Some(id) = Id::new_checked(after.parse()?) {
            id
        } else {
            return Err(anyhow::anyhow!("invalid 'after' message id"));
        };

        req.after(message_id).await
    } else {
        req.await
    };

    let messages = res
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(messages.into_iter().map(Into::into).collect())
}

#[op2(async)]
#[serde]
pub async fn op_discord_create_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpCreateChannelMessage,
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
        .create_message(channel.id)
        .embeds(&maybe_embeds)?
        .components(&components)?;

    if let Some(content) = &args.fields.content {
        mc = mc.content(content)?
    }

    let mentions = args.fields.allowed_mentions.map(Into::into);
    if mentions.is_some() {
        mc = mc.allowed_mentions(mentions.as_ref());
    }

    Ok(mc
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_edit_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpEditChannelMessage,
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
        .update_message(channel.id, message_id.cast())
        .content(args.fields.content.as_deref())?
        .components(components.as_deref())?;

    if let Some(embeds) = &maybe_embeds {
        mc = mc.embeds(Some(embeds))?;
    }

    let mentions = args.fields.allowed_mentions.map(Into::into);
    if mentions.is_some() {
        mc = mc.allowed_mentions(mentions.as_ref());
    }

    Ok(mc
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
pub async fn op_discord_crosspost_message(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] message_id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let ctx = get_rt_ctx(&state);
    get_guild_channel(&state, &ctx, channel_id).await?;

    ctx.discord_config
        .client
        .crosspost_message(channel_id, message_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_interaction_callback(
    state: Rc<RefCell<OpState>>,
    #[serde] args: InteractionCallback,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    client
        .create_response(
            Id::from_str(&args.interaction_id)?,
            &args.interaction_token,
            &args.data.into(),
        )
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_interaction_get_original_response(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    Ok(client
        .response(&token)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_interaction_edit_original_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpCreateFollowUpMessage,
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
        .update_response(&args.interaction_token)
        .content(args.fields.content.as_deref())?
        .embeds(maybe_embeds.as_deref())?
        .components(components.as_deref())?
        .content(args.fields.content.as_deref())?;

    let mentions = args.fields.allowed_mentions.map(Into::into);
    if mentions.is_some() {
        mc = mc.allowed_mentions(mentions.as_ref());
    }

    Ok(mc
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
pub async fn op_discord_interaction_delete_original(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    client.delete_response(&token).await?;
    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_interaction_get_followup_message(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
    #[serde] id: Id<MessageMarker>,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    Ok(client
        .followup(&token, id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_interaction_followup_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpCreateFollowUpMessage,
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
        .create_followup(&args.interaction_token)
        .embeds(&maybe_embeds)?
        .components(&components)?;

    if let Some(flags) = args.flags {
        mc = mc.flags(flags.into());
    }

    if let Some(content) = &args.fields.content {
        mc = mc.content(content)?
    }

    let mentions = args.fields.allowed_mentions.map(Into::into);
    if mentions.is_some() {
        mc = mc.allowed_mentions(mentions.as_ref());
    }

    Ok(mc
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_interaction_edit_followup_message(
    state: Rc<RefCell<OpState>>,
    #[serde] message_id: Id<MessageMarker>,
    #[serde] args: OpCreateFollowUpMessage,
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
        .update_followup(&args.interaction_token, message_id)
        .content(args.fields.content.as_deref())?
        .embeds(maybe_embeds.as_deref())?
        .components(components.as_deref())?
        .content(args.fields.content.as_deref())?;

    let mentions = args.fields.allowed_mentions.map(Into::into);
    if mentions.is_some() {
        mc = mc.allowed_mentions(mentions.as_ref());
    }

    mc.await.map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_interaction_delete_followup_message(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
    #[serde] id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let client = rt_ctx.discord_config.interaction_client();
    client
        .delete_followup(&token, id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;
    Ok(())
}

#[op2(async)]
pub async fn op_discord_delete_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpDeleteMessage,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &args.channel_id).await?;
    let message_id = parse_str_snowflake_id(&args.message_id)?;

    rt_ctx
        .discord_config
        .client
        .delete_message(channel.id, message_id.cast())
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_bulk_delete_messages(
    state: Rc<RefCell<OpState>>,
    #[serde] args: OpDeleteMessagesBulk,
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
        .delete_messages(channel.id, &message_ids)?
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Roles
#[op2(async)]
#[serde]
pub async fn op_discord_get_role(
    state: Rc<RefCell<OpState>>,
    #[serde] role_id: Id<RoleMarker>,
) -> Result<runtime_models::discord::role::Role, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    match rt_ctx.bot_state.get_role(rt_ctx.guild_id, role_id).await? {
        Some(c) => Ok(c.into()),
        _ => Err(not_found_error(format!("role `{role_id}` not found"))),
    }
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_roles(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::discord::role::Role>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let roles = rt_ctx.bot_state.get_roles(rt_ctx.guild_id).await?;
    Ok(roles.into_iter().map(Into::into).collect())
}

// Reactions
#[op2(async)]
pub async fn op_discord_create_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    #[serde] emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .create_reaction(channel_id, message_id, &(&emoji).into())
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_delete_own_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    #[serde] emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_current_user_reaction(channel_id, message_id, &(&emoji).into())
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_delete_user_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] (channel_id, message_id, user_id): (
        Id<ChannelMarker>,
        Id<MessageMarker>,
        Id<UserMarker>,
    ),
    #[serde] emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the ctx guild
    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_reaction(channel_id, message_id, &(&emoji).into(), user_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_reactions(
    state: Rc<RefCell<OpState>>,
    #[serde] (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    #[serde] fields: GetReactionsFields,
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
        req = req.limit(limit as u16)?;
    }

    Ok(req
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[op2(async)]
pub async fn op_discord_delete_all_reactions(
    state: Rc<RefCell<OpState>>,
    #[serde] (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_all_reactions(channel_id, message_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_delete_all_reactions_for_emoji(
    state: Rc<RefCell<OpState>>,
    #[serde] (channel_id, message_id): (Id<ChannelMarker>, Id<MessageMarker>),
    #[serde] emoji: SendEmoji,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let _ = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_all_reaction(channel_id, message_id, &(&emoji).into())
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Channels
#[op2(async)]
#[serde]
pub async fn op_discord_get_channel(
    state: Rc<RefCell<OpState>>,
    #[string] channel_id_str: String,
) -> Result<runtime_models::internal::channel::GuildChannel, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channel = parse_get_guild_channel(&state, &rt_ctx, &channel_id_str).await?;
    Ok(channel.into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_channels(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::internal::channel::GuildChannel>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channels = rt_ctx.bot_state.get_channels(rt_ctx.guild_id).await?;
    Ok(channels.into_iter().map(Into::into).collect())
}

#[op2(async)]
#[serde]
pub async fn op_discord_edit_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] params: EditChannel,
) -> Result<runtime_models::internal::channel::GuildChannel, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the channel exists on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    let mut overwrites = Vec::new();
    let edit = rt_ctx.discord_config.client.update_channel(channel_id);

    Ok(params
        .apply(&mut overwrites, edit)?
        .await?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_create_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] params: CreateChannel,
) -> Result<runtime_models::internal::channel::GuildChannel, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let mut overwrites = Vec::new();
    let edit = rt_ctx
        .discord_config
        .client
        .create_guild_channel(rt_ctx.guild_id, &params.name)?;

    Ok(params
        .apply(&mut overwrites, edit)?
        .await?
        .model()
        .await?
        .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_delete_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
) -> Result<runtime_models::internal::channel::GuildChannel, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the channel exists on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    Ok(rt_ctx
        .discord_config
        .client
        .delete_channel(channel_id)
        .await?
        .model()
        .await?
        .into())
}

#[op2(async)]
pub async fn op_discord_update_channel_permission(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] permission_overwrite: PermissionOverwrite,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the channel exists on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    let conv = permission_overwrite
        .try_into()
        .map_err(|_| anyhow!("invalid id"))?;

    rt_ctx
        .discord_config
        .client
        .update_channel_permission(channel_id, &conv)
        .await?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_delete_channel_permission(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] (kind, overwrite_id): (PermissionOverwriteType, Id<GenericMarker>),
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the channel exists on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    let req = rt_ctx
        .discord_config
        .client
        .delete_channel_permission(channel_id);

    match kind {
        PermissionOverwriteType::Member => req.member(overwrite_id.cast()).await?,
        PermissionOverwriteType::Role => req.role(overwrite_id.cast()).await?,
    };

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_channel_invites(
    state: Rc<RefCell<OpState>>,
    #[string] channel_id_str: String,
) -> Result<Vec<runtime_models::internal::invite::Invite>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    let channel = parse_get_guild_channel(&state, &rt_ctx, &channel_id_str).await?;

    let rt_ctx = get_rt_ctx(&state);

    let resp = rt_ctx
        .discord_config
        .client
        .channel_invites(channel.id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    resp.into_iter().map(TryInto::try_into).collect()
}

#[op2(async)]
#[serde]
pub async fn op_discord_create_channel_invite(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] fields: CreateInviteFields,
) -> Result<runtime_models::internal::invite::Invite, AnyError> {
    let rt_ctx = get_rt_ctx(&state);
    let channel = get_guild_channel(&state, &rt_ctx, channel_id).await?;

    let mut req = rt_ctx.discord_config.client.create_invite(channel.id);

    if let Some(max_age) = fields.max_age {
        req = req.max_age(max_age)?;
    }
    if let Some(max_uses) = fields.max_uses {
        req = req.max_uses(max_uses)?;
    }
    if let Some(temporary) = fields.temporary {
        req = req.temporary(temporary);
    }
    if let Some(target_application_id) = fields.target_application_id {
        req = req.target_application_id(target_application_id);
    }
    if let Some(target_user_id) = fields.target_user_id {
        req = req.target_user_id(target_user_id);
    }
    if let Some(target_type) = fields.target_type {
        req = req.target_type(target_type.into());
    }
    if let Some(unique) = fields.unique {
        req = req.unique(unique);
    }

    req.await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?
        .try_into()
}

// Pins
#[op2(async)]
#[serde]
pub async fn op_discord_get_channel_pins(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
) -> Result<Vec<Message>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    let pins = rt_ctx
        .discord_config
        .client
        .pins(channel_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(pins.into_iter().map(Into::into).collect())
}

#[op2(async)]
pub async fn op_discord_create_pin(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] message_id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .create_pin(channel_id, message_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_delete_pin(
    state: Rc<RefCell<OpState>>,
    #[serde] channel_id: Id<ChannelMarker>,
    #[serde] message_id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    // ensure the provided channel is on the guild
    get_guild_channel(&state, &rt_ctx, channel_id).await?;

    rt_ctx
        .discord_config
        .client
        .delete_pin(channel_id, message_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Members
#[op2(async)]
#[serde]
pub async fn op_discord_get_members(
    state: Rc<RefCell<OpState>>,
    #[serde] user_ids: Vec<String>,
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

    let valid_ids = ids.iter().filter(|v| v.is_some()).count();
    if valid_ids > 2 {
        info!("Fetching members through gateway");

        let resp = rt_ctx
            .bot_state
            .get_guild_members(rt_ctx.guild_id, ids.iter().filter_map(|v| *v).collect())
            .await?;

        let mut ret = Vec::with_capacity(ids.len());
        for id in ids {
            match id {
                Some(user_id) => ret.push(
                    resp.iter()
                        .find(|v| v.user.id == user_id)
                        .cloned()
                        .map(From::from),
                ),
                None => ret.push(None),
            }
        }

        Ok(ret)
    } else {
        Ok(fetch_members_through_api(&state, &rt_ctx, ids).await?)
    }
}

async fn fetch_members_through_api(
    state: &Rc<RefCell<OpState>>,
    rt_ctx: &RuntimeContext,
    ids: Vec<Option<Id<UserMarker>>>,
) -> Result<Vec<Option<runtime_models::internal::member::Member>>, AnyError> {
    let mut res = Vec::new();
    for item in ids {
        if let Some(id) = item {
            // fall back to http api
            match rt_ctx
                .discord_config
                .client
                .guild_member(rt_ctx.guild_id, id)
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
                        return Err(handle_discord_error(state, err));
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

#[op2(async)]
pub async fn op_discord_add_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    rt_ctx
        .discord_config
        .client
        .add_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_remove_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    rt_ctx
        .discord_config
        .client
        .remove_guild_member_role(rt_ctx.guild_id, user_id, role_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_update_member(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] fields: UpdateGuildMemberFields,
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

    if let Some(ts) = &fields.communication_disabled_until {
        builder = builder.communication_disabled_until(
            ts.map(|v| twilight_model::util::Timestamp::from_micros(v.0 as i64 * 1000))
                .transpose()?,
        )?;
    }

    let ret = builder
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;
    Ok(ret.into())
}

// Bans
#[op2(async)]
pub async fn op_discord_create_ban(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] extras: CreateBanFields,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let mut req = rt_ctx
        .discord_config
        .client
        .create_ban(rt_ctx.guild_id, user_id);

    if let Some(days) = extras.delete_message_days {
        req = req.delete_message_seconds(days * 24 * 60 * 60)?;
    }

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.await.map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_ban(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
) -> Result<Ban, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let result = rt_ctx
        .discord_config
        .client
        .ban(rt_ctx.guild_id, user_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(result.into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_bans(state: Rc<RefCell<OpState>>) -> Result<Vec<Ban>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let result = rt_ctx
        .discord_config
        .client
        .bans(rt_ctx.guild_id)
        .await
        .map_err(|err| handle_discord_error(&state, err))?
        .model()
        .await?;

    Ok(result.into_iter().map(Into::into).collect())
}

#[op2(async)]
pub async fn op_discord_delete_ban(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] extras: AuditLogExtras,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let mut req = rt_ctx
        .discord_config
        .client
        .delete_ban(rt_ctx.guild_id, user_id);

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.await.map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

// Other
#[op2(async)]
pub async fn op_discord_remove_member(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] extras: AuditLogExtras,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let mut req = rt_ctx
        .discord_config
        .client
        .remove_guild_member(rt_ctx.guild_id, user_id);

    if let Some(reason) = &extras.audit_log_reason {
        req = req.reason(reason)?;
    }

    req.await.map_err(|err| handle_discord_error(&state, err))?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_member_permissions(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] (roles, channel_id): (Option<Vec<Id<RoleMarker>>>, Option<Id<ChannelMarker>>),
) -> Result<(String, Option<String>), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let member_roles = if let Some(roles) = roles {
        roles
    } else {
        let member = rt_ctx
            .discord_config
            .client
            .guild_member(rt_ctx.guild_id, user_id)
            .await?
            .model()
            .await?;

        member.roles
    };

    let guild_roles = rt_ctx.bot_state.get_roles(rt_ctx.guild_id).await?;
    let guild = if let Some(guild) = rt_ctx.bot_state.get_guild(rt_ctx.guild_id).await? {
        guild
    } else {
        return Err(anyhow!("guild not in state"));
    };

    let role_perms_pair = member_roles
        .iter()
        .filter_map(|rid| {
            guild_roles
                .iter()
                .find(|r| r.id == *rid)
                .map(|r| (*rid, r.permissions))
        })
        .collect::<Vec<_>>();

    let everyone_role = guild_roles
        .iter()
        .find(|v| v.id == rt_ctx.guild_id.cast::<RoleMarker>())
        .map(|v| v.permissions)
        .unwrap_or(Permissions::empty());

    let calc = twilight_util::permission_calculator::PermissionCalculator::new(
        rt_ctx.guild_id,
        user_id,
        everyone_role,
        role_perms_pair.as_slice(),
    )
    .owner_id(guild.owner_id);

    let guild_perms = calc.root();
    let channel_perms = if let Some(channel_id) = channel_id {
        let channel = get_guild_channel(&state, &rt_ctx, channel_id).await?;
        // match channel.
        match channel.kind {
            twilight_model::channel::ChannelType::AnnouncementThread
            | twilight_model::channel::ChannelType::PublicThread
            | twilight_model::channel::ChannelType::PrivateThread => {
                let real_channel = get_guild_channel(
                    &state,
                    &rt_ctx,
                    channel
                        .parent_id
                        .ok_or_else(|| anyhow!("thread has no parent??"))?,
                )
                .await?;

                Some(
                    calc.in_channel(
                        real_channel.kind,
                        real_channel
                            .permission_overwrites
                            .as_deref()
                            .unwrap_or_default(),
                    ),
                )
            }
            _ => Some(calc.in_channel(
                channel.kind,
                channel.permission_overwrites.as_deref().unwrap_or_default(),
            )),
        }
    } else {
        None
    };

    Ok((
        guild_perms.bits().to_string(),
        channel_perms.map(|v| v.bits().to_string()),
    ))
}
