use anyhow::anyhow;
use chrono::TimeZone;
use common::DiscordConfig;
use deno_core::{
    error::{custom_error, get_custom_error_class},
    op2, OpState,
};
use futures::TryFutureExt;
use pin_project::pin_project;
use runtime_models::{
    discord::{
        channel::{PermissionOverwrite, PermissionOverwriteType},
        guild::Guild,
        message::SendEmoji,
        role::Role,
        util::AuditLogExtras,
    },
    internal::{
        channel::{
            CreateChannel, CreateForumThread, CreateThread, CreateThreadFromMessage, EditChannel,
            EditGuildChannelPosition, ForumThreadResponse, GuildChannel, ListThreadMembersRequest,
            ListThreadsRequest, ThreadMember, ThreadsListing, UpdateThread,
        },
        emoji::{CustomEmoji, OpCreateEmoji, OpUpdateEmoji},
        events::VoiceState,
        interactions::InteractionCallback,
        invite::CreateInviteFields,
        member::{Ban, UpdateGuildMemberFields},
        messages::{
            convert_attachments, Message, OpCreateChannelMessage, OpCreateFollowUpMessage,
            OpDeleteMessage, OpDeleteMessagesBulk, OpEditChannelMessage, OpGetMessages,
        },
        misc_op::{CreateBanFields, GetReactionsFields},
        role::{OpCreateRoleFields, OpUpdateRoleFields, UpdateRolePosition},
        user::User,
        webhook::{
            DiscordWebhook, OpCreateWebhook, OpEditWebhook, OpEditWebhookWithToken,
            OpExecuteWebhook, OpUpdateWebhookMessage, OpWebhookMessageSpecifier,
            OpWebhookSpecifier,
        },
    },
    ops::{handle_async_op, EasyOpsASync, EasyOpsHandlerASync},
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::VecDeque,
    fmt::{Display, Formatter},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use std::{cell::RefCell, rc::Rc};
use tokio::runtime::Handle;
use tracing::{info, warn};
use twilight_http::error::ErrorType;
use twilight_http::request::AuditLogReason;
use twilight_http::{
    api_error::{ApiError, GeneralApiError},
    response::StatusCode,
};
use twilight_model::id::Id;
use twilight_model::{
    guild::Permissions,
    id::marker::{ChannelMarker, UserMarker},
};
use twilight_model::{
    guild::RolePosition,
    id::marker::{
        GenericMarker, InteractionMarker, MessageMarker, RoleMarker, TagMarker, WebhookMarker,
    },
};
use vm::AnyError;

use super::{get_guild_channel, parse_discord_id, parse_get_guild_channel, parse_str_snowflake_id};
use crate::{
    extensions::parse_str_snowflake_ids, get_rt_ctx, limits::RateLimiters, RuntimeContext,
};

deno_core::extension!(
    bl_discord,
    ops = [
        // guild
        // op_discord_get_guild,
        op_discord_get_invites,
        op_discord_get_invite,
        op_discord_delete_invite,
        // messages
        // op_discord_get_message,
        // op_discord_get_messages,
        // op_discord_create_message,
        // op_discord_edit_message,
        // op_discord_crosspost_message,
        // op_discord_delete_message,
        // op_discord_bulk_delete_messages,
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
        op_easyops_async,
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

#[pin_project]
pub struct WrappedFuture<Fut>
where
    Fut: Future,
{
    #[pin]
    inner: Fut,
    rt: Handle,
}

impl<Fut> Future for WrappedFuture<Fut>
where
    Fut: Future,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        let _guard = this.rt.enter();
        let res = this.inner.poll(cx);
        drop(_guard);
        res
    }
}

pub async fn discord_request_retry<T: Send, Fut>(
    state: &Rc<RefCell<OpState>>,
    f: impl Fn(Arc<DiscordConfig>) -> Fut,
) -> Result<T, AnyError>
where
    Fut: Future<Output = Result<T, AnyError>>,
{
    let rt_handle = {
        let state = state.borrow();
        let rt_ctx: &RuntimeContext = state.borrow();
        rt_ctx.main_tokio_runtime.clone()
    };

    let mut retry_sleep = Duration::from_secs(1);
    let max_sleep = Duration::from_secs(60);
    let mut retry_count = 1;

    let res = loop {
        let discord_config = {
            let state = state.borrow();
            let rt_ctx: &RuntimeContext = state.borrow();
            rt_ctx.discord_config.clone()
        };

        // We run the request on the main botloader tokio runtime, the vm runtime is disposed of on shutdown
        // and any futures cancelled, which twilights ratelimit handling does not like
        let inner_fut = f(discord_config);
        let fut = WrappedFuture {
            inner: inner_fut,
            rt: rt_handle.clone(),
        };

        let retry_reason = match fut.await {
            Ok(v) => break Ok(v),
            Err(err) => match err.downcast::<twilight_http::Error>() {
                Ok(discord_err) => match discord_err.kind() {
                    ErrorType::RequestCanceled => "REQUEST_CANCELED",
                    ErrorType::RequestError => "REQUEST_ERROR",

                    // I've disabled this for now, from experience this could lead to requests being processed twice.
                    // ErrorType::RequestTimedOut => "REQUEST_TIMEOUT",
                    ErrorType::Response { status, .. } if status.get() == 429 => {
                        if check_bad_request(state, status.get()).is_err() {
                            break Err(anyhow::anyhow!(
                                "Hit maximum number of bad requests within a time period"
                            ));
                        }

                        "RATELIMIT_429"
                    }
                    ErrorType::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
                    _ => break Err(handle_discord_error(state, discord_err)),
                },
                Err(err) => break Err(err),
            },
        };
        tokio::time::sleep(retry_sleep).await;
        retry_sleep = retry_sleep * 2;
        if retry_sleep > max_sleep {
            retry_sleep = max_sleep;
        }
        retry_count += 1;

        warn!("Retrying request, reason: {retry_reason}, count: {retry_count}");
    };

    res
}

pub async fn discord_request_any<T: Send + Sync + 'static>(
    state: &Rc<RefCell<OpState>>,
    f: impl Future<Output = Result<T, AnyError>> + Send + 'static,
) -> Result<T, AnyError> {
    let rt_handle = {
        let state = state.borrow();
        let rt_ctx: &RuntimeContext = state.borrow();
        rt_ctx.main_tokio_runtime.clone()
    };

    rt_handle
        .spawn(f)
        .await
        .unwrap()
        .map_err(|err| match err.downcast::<twilight_http::Error>() {
            Ok(discord_err) => handle_discord_error(state, discord_err),
            Err(err) => err,
        })
}

pub async fn discord_request<T: Send + Sync + 'static>(
    state: &Rc<RefCell<OpState>>,
    f: impl Future<Output = Result<T, twilight_http::Error>> + Send + 'static,
) -> Result<T, AnyError> {
    let rt_handle = {
        let state = state.borrow();
        let rt_ctx: &RuntimeContext = state.borrow();
        rt_ctx.main_tokio_runtime.clone()
    };

    rt_handle
        .spawn(f)
        .await
        .unwrap()
        .map_err(|err| handle_discord_error(state, err))
}

pub async fn discord_request_with_extra_error<T: Send + Sync + 'static>(
    state: &Rc<RefCell<OpState>>,
    f: impl Future<Output = Result<Result<T, twilight_http::Error>, AnyError>> + Send + 'static,
) -> Result<T, AnyError> {
    let rt_handle = {
        let state = state.borrow();
        let rt_ctx: &RuntimeContext = state.borrow();
        rt_ctx.main_tokio_runtime.clone()
    };

    rt_handle
        .spawn(f)
        .await
        .unwrap()?
        .map_err(|err| handle_discord_error(state, err))
}

fn check_bad_request(state: &Rc<RefCell<OpState>>, code: u16) -> Result<(), ()> {
    if code == 401 || code == 403 || code == 429 {
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
            handle.shutdown_vm(
                vm::vm::ShutdownReason::DiscordInvalidRequestsRatelimit,
                false,
            );

            return Err(());
        }
    }

    Ok(())
}

pub fn handle_discord_error(state: &Rc<RefCell<OpState>>, err: twilight_http::Error) -> AnyError {
    let kind = err.kind();
    if let ErrorType::Response { status, .. } = kind {
        // check if this guild has run into a lot of "invalid" requests
        //
        // this is needed because discord will ban our IP if we exceed 10_000 invalid req/10min as of writing
        let raw = status.get();
        let _ = check_bad_request(state, raw);
    }

    match kind {
        ErrorType::Response {
            // 10008 is unknown message
            error: ApiError::General(GeneralApiError { code, message, .. }),
            status,
            body,
        } => error_from_code(*status, *code, message, body),
        _ => err.into(),
    }
}

pub fn error_from_code(
    resp_code: StatusCode,
    code: u64,
    message: &str,
    body: &Vec<u8>,
) -> AnyError {
    match resp_code.get() {
        404 => not_found_error(format!("{code}: {message}")),
        403 => custom_error("DiscordPermissionsError", format!("{code}: {message}")),
        400..=499 => match code {
            30001..=40000 => custom_error("DiscordLimitReachedError", format!("{code}: {message}")),
            50035 => {
                if let Ok(parsed) = serde_json::from_slice::<BlDiscordApiError>(&body) {
                    if let Some(errors) = parsed.errors {
                        return custom_error(
                            "DiscordFormError",
                            format!("{code}: {message} - {}", errors),
                        );
                    }
                }

                custom_error("DiscordGenericErrorResponse", format!("{code}: {message}"))
            }
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

const DISCORD_NOT_FOUND_CLASS_NAME: &str = "DiscordNotFoundError";

pub fn not_found_error(message: impl Into<Cow<'static, str>>) -> AnyError {
    custom_error(DISCORD_NOT_FOUND_CLASS_NAME, message)
}

#[op2(async)]
#[serde]
pub async fn op_easyops_async(
    state: Rc<RefCell<OpState>>,
    #[serde] op: EasyOpsASync,
) -> Result<serde_json::Value, AnyError> {
    let handler = EasyOpsHandler { state };
    handle_async_op(&handler, op).await
}

struct EasyOpsHandler {
    state: Rc<RefCell<OpState>>,
}

impl EasyOpsHandlerASync for EasyOpsHandler {
    async fn discord_get_guild(&self, _arg: ()) -> Result<Guild, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

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

    async fn discord_get_message(
        &self,
        (channel_id_raw, message_id_raw): (String, String),
    ) -> Result<Message, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &channel_id_raw).await?;
        let message_id = parse_discord_id(&message_id_raw)?;

        discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .message(channel.id, message_id)
                .await
        })
        .await?
        .model()
        .await?
        .try_into()
    }

    async fn discord_get_messages(
        &self,
        args: OpGetMessages,
    ) -> Result<Vec<Message>, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &args.channel_id).await?;

        let limit = if let Some(limit) = args.limit {
            limit.clamp(1, 100)
        } else {
            50
        };

        let before_id: Option<Id<MessageMarker>> = args
            .before
            .map(|v| parse_discord_id(&v))
            .transpose()
            .map_err(|err| anyhow::anyhow!("invalid 'before' message id: {err}"))?;

        let after_id: Option<Id<MessageMarker>> = args
            .after
            .map(|v| parse_discord_id(&v))
            .transpose()
            .map_err(|err| anyhow::anyhow!("invalid 'after' message id: {err}"))?;

        let res = discord_request_with_extra_error(&self.state, async move {
            let req = rt_ctx
                .discord_config
                .client
                .channel_messages(channel.id)
                .limit(limit as u16);

            if let Some(before) = before_id {
                Ok(req.before(before).await)
            } else if let Some(after) = after_id {
                Ok(req.after(after).await)
            } else {
                Ok(req.await)
            }
        })
        .await?;

        let messages = res.model().await?;

        messages
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
    }

    async fn discord_create_message(
        &self,
        args: OpCreateChannelMessage,
    ) -> Result<Message, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &args.channel_id).await?;

        let attachments = convert_attachments(args.fields.attachments.unwrap_or_default())?;

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
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let mentions: Option<twilight_model::channel::message::AllowedMentions> =
            args.fields.allowed_mentions.map(Into::into);

        discord_request_retry(&self.state, |discord_config| async {
            let conf = discord_config;
            let mut mc = conf
                .client
                .create_message(channel.id)
                .embeds(&maybe_embeds)
                .components(&components);

            if let Some(content) = &args.fields.content {
                mc = mc.content(content)
            }

            if mentions.is_some() {
                mc = mc.allowed_mentions(mentions.as_ref());
            }

            if let Some(reply) = &args.fields.reply_to_message_id {
                mc = mc.reply(parse_discord_id(reply)?);
            }

            if attachments.len() > 0 {
                mc = mc.attachments(&attachments);
            }

            Ok(mc.await?)
        })
        .await?
        .model()
        .await?
        .try_into()
    }

    async fn discord_edit_message(
        &self,
        args: OpEditChannelMessage,
    ) -> Result<Message, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &args.channel_id).await?;
        let message_id = parse_str_snowflake_id(&args.message_id)?;
        let attachments = convert_attachments(args.fields.attachments.unwrap_or_default())?;

        let res = discord_request_with_extra_error(&self.state, async move {
            let maybe_embeds = args
                .fields
                .embeds
                .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

            let components = args
                .fields
                .components
                .map(|inner| {
                    inner
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;

            let mut mc = rt_ctx
                .discord_config
                .client
                .update_message(channel.id, message_id.cast())
                .content(args.fields.content.as_deref())
                .components(components.as_deref());

            if let Some(embeds) = &maybe_embeds {
                mc = mc.embeds(Some(embeds));
            }

            let mentions = args.fields.allowed_mentions.map(Into::into);
            if mentions.is_some() {
                mc = mc.allowed_mentions(mentions.as_ref());
            }

            if !attachments.is_empty() {
                mc = mc.attachments(&attachments);
            }

            Ok(mc.await)
        })
        .await?;

        res.model().await?.try_into()
    }

    async fn discord_crosspost_message(
        &self,
        (channel_id_raw, message_id_raw): (String, String),
    ) -> Result<(), anyhow::Error> {
        let ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &ctx, &channel_id_raw).await?;
        let message_id: Id<MessageMarker> = parse_discord_id(&message_id_raw)?;

        discord_request(&self.state, async move {
            ctx.discord_config
                .client
                .crosspost_message(channel.id, message_id)
                .await
        })
        .await?;

        Ok(())
    }

    async fn discord_delete_message(&self, args: OpDeleteMessage) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &args.channel_id).await?;
        let message_id = parse_str_snowflake_id(&args.message_id)?;
        discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .delete_message(channel.id, message_id.cast())
                .await
        })
        .await?;

        Ok(())
    }

    async fn discord_bulk_delete_messages(
        &self,
        args: OpDeleteMessagesBulk,
    ) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &args.channel_id).await?;
        let message_ids = args
            .message_ids
            .iter()
            .filter_map(|v| parse_str_snowflake_id(v).ok())
            .map(|v| v.cast())
            .collect::<Vec<_>>();

        discord_request_with_extra_error(&self.state, async move {
            Ok(rt_ctx
                .discord_config
                .client
                .delete_messages(channel.id, &message_ids)
                .await)
        })
        .await?;

        Ok(())
    }

    async fn discord_start_thread_from_message(
        &self,
        arg: CreateThreadFromMessage,
    ) -> Result<GuildChannel, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;
        let message_id = parse_discord_id(&arg.message_id)?;

        Ok(discord_request_with_extra_error(&self.state, async move {
            let mut req = rt_ctx
                .discord_config
                .client
                .create_thread_from_message(channel.id, message_id, &arg.name);

            if let Some(auto_archive) = arg.auto_archive_duration_minutes {
                req = req.auto_archive_duration(
                    twilight_model::channel::thread::AutoArchiveDuration::from(auto_archive),
                )
            }

            Ok(req.await)
        })
        .await?
        .model()
        .await?
        .try_into()?)
    }

    async fn discord_start_thread_without_message(
        &self,
        arg: CreateThread,
    ) -> Result<GuildChannel, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;

        Ok(discord_request_with_extra_error(&self.state, async move {
            let mut req =
                rt_ctx
                    .discord_config
                    .client
                    .create_thread(channel.id, &arg.name, arg.kind.into());

            if let Some(auto_archive) = arg.auto_archive_duration_minutes {
                req = req.auto_archive_duration(
                    twilight_model::channel::thread::AutoArchiveDuration::from(auto_archive),
                )
            }

            if let Some(invitable) = arg.invitable {
                req = req.invitable(invitable);
            }

            Ok(req.await)
        })
        .await?
        .model()
        .await?
        .try_into()?)
    }

    async fn discord_start_forum_thread(
        &self,
        arg: CreateForumThread,
    ) -> Result<ForumThreadResponse, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;

        let res = discord_request_with_extra_error(&self.state, async move {
            let mut req = rt_ctx
                .discord_config
                .client
                .create_forum_thread(channel.id, &arg.name);

            if let Some(auto_archive) = arg.auto_archive_duration_minutes {
                req = req.auto_archive_duration(
                    twilight_model::channel::thread::AutoArchiveDuration::from(auto_archive),
                )
            }

            let maybe_tags: Option<Vec<Id<TagMarker>>> = arg.tag_ids.map(|v| {
                v.into_iter()
                    .filter_map(|string_id| parse_discord_id(&string_id).ok())
                    .collect()
            });

            if let Some(tags) = &maybe_tags {
                req = req.applied_tags(tags);
            }

            // Can't find ratelimit usage
            // if let Some(ratelimit) = arg.rate_limit_per_user{
            //     req = req.
            // }

            let mut req = req.message();

            let embeds = arg
                .message
                .embeds
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();

            if !embeds.is_empty() {
                req = req.embeds(&embeds);
            }

            let components = arg
                .message
                .components
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?;

            if !components.is_empty() {
                req = req.components(&components);
            }

            if let Some(content) = &arg.message.content {
                req = req.content(content);
            }

            let mentions = arg.message.allowed_mentions.map(Into::into);
            if mentions.is_some() {
                req = req.allowed_mentions(mentions.as_ref());
            }

            Ok(req.await)
        })
        .await?;

        let result = res.model().await?;

        Ok(ForumThreadResponse {
            message: result.message.try_into()?,
            channel: result.channel.try_into()?,
        })
    }

    async fn discord_add_thread_member(
        &self,
        (channel_id_raw, user_id_raw): (String, String),
    ) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &channel_id_raw).await?;
        let user_id: Id<UserMarker> = parse_discord_id(&user_id_raw)?;

        discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .add_thread_member(channel.id, user_id)
                .await
        })
        .await?;

        Ok(())
    }

    async fn discord_remove_thread_member(
        &self,
        (channel_id_raw, user_id_raw): (String, String),
    ) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &channel_id_raw).await?;
        let user_id: Id<UserMarker> = parse_discord_id(&user_id_raw)?;

        discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .remove_thread_member(channel.id, user_id)
                .await
        })
        .await?;

        Ok(())
    }

    async fn discord_list_thread_members(
        &self,
        args: ListThreadMembersRequest,
    ) -> Result<Vec<ThreadMember>, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &args.channel_id).await?;

        Ok(discord_request_with_extra_error(&self.state, async move {
            let mut req = rt_ctx.discord_config.client.thread_members(channel.id);
            if let Some(limit) = args.limit {
                req = req.limit(limit);
            }
            if let Some(after) = args.after_user_id {
                req = req.after(parse_discord_id(&after)?);
            }
            if let Some(with_member) = args.with_member {
                req = req.with_member(with_member);
            }

            Ok(req.await)
        })
        .await?
        .models()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    async fn discord_list_active_threads(&self, _arg: ()) -> Result<ThreadsListing, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        Ok(discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .active_threads(rt_ctx.guild_id)
                .await
        })
        .await?
        .model()
        .await?
        .try_into()?)
    }

    async fn discord_list_public_archived_threads(
        &self,
        arg: ListThreadsRequest,
    ) -> Result<ThreadsListing, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;

        let before_str = arg
            .before
            .map(|v| {
                chrono::Utc
                    .timestamp_millis_opt(v.0 as i64)
                    .single()
                    .ok_or(anyhow!("bad 'before' timestamp"))
                    .map(|ts| ts.to_rfc3339())
            })
            .transpose()?;

        Ok(discord_request(&self.state, async move {
            let mut threads_request = rt_ctx
                .discord_config
                .client
                .public_archived_threads(channel.id);

            if let Some(before_str) = &before_str {
                threads_request = threads_request.before(before_str);
            }

            threads_request.await
        })
        .await?
        .model()
        .await?
        .try_into()?)
    }

    async fn discord_list_private_archived_threads(
        &self,
        arg: ListThreadsRequest,
    ) -> Result<ThreadsListing, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;

        let before_str = arg
            .before
            .map(|v| {
                chrono::Utc
                    .timestamp_millis_opt(v.0 as i64)
                    .single()
                    .ok_or(anyhow!("bad 'before' timestamp"))
                    .map(|ts| ts.to_rfc3339())
            })
            .transpose()?;

        Ok(discord_request(&self.state, async move {
            let mut threads_request = rt_ctx
                .discord_config
                .client
                .private_archived_threads(channel.id);

            if let Some(before_str) = &before_str {
                threads_request = threads_request.before(before_str);
            }

            threads_request.await
        })
        .await?
        .model()
        .await?
        .try_into()?)
    }

    async fn discord_edit_thread(&self, arg: UpdateThread) -> Result<GuildChannel, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;

        Ok(discord_request_with_extra_error(&self.state, async move {
            let mut req = rt_ctx.discord_config.client.update_thread(channel.id);

            let maybe_tags: Option<Vec<Id<TagMarker>>> = arg.tag_ids.map(|v| {
                v.into_iter()
                    .filter_map(|string_id| parse_discord_id(&string_id).ok())
                    .collect()
            });

            if let Some(tags) = &maybe_tags {
                req = req.applied_tags(Some(tags));
            }

            if let Some(archived) = arg.archived {
                req = req.archived(archived);
            }

            if let Some(auto_archive_duration_minutes) = arg.auto_archive_duration_minutes {
                req = req.auto_archive_duration(auto_archive_duration_minutes.into())
            }

            if let Some(invitable) = arg.invitable {
                req = req.invitable(invitable)
            }

            if let Some(locked) = arg.locked {
                req = req.locked(locked)
            }

            if let Some(name) = &arg.name {
                req = req.name(name);
            }

            if let Some(rate_limit_per_user) = &arg.rate_limit_per_user {
                req = req.rate_limit_per_user(*rate_limit_per_user);
            }

            Ok(req.await)
        })
        .await?
        .model()
        .await?
        .try_into()?)
    }

    async fn discord_bulk_edit_channels(
        &self,
        arg: Vec<EditGuildChannelPosition>,
    ) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        discord_request_with_extra_error(&self.state, async move {
            let positions = arg
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<_>, _>>()?;

            let req = rt_ctx
                .discord_config
                .client
                .update_guild_channel_positions(rt_ctx.guild_id, &positions);

            Ok(req.await)
        })
        .await?;

        Ok(())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_create_role(&self, arg: OpCreateRoleFields) -> Result<Role, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let created = discord_request_with_extra_error(&self.state, async move {
            let mut create_role = rt_ctx.discord_config.client.create_role(rt_ctx.guild_id);

            if let Some(color) = arg.color {
                create_role = create_role.color(color);
            }
            if let Some(hoist) = arg.hoist {
                create_role = create_role.hoist(hoist);
            }
            if let Some(mentionable) = arg.mentionable {
                create_role = create_role.mentionable(mentionable);
            }
            if let Some(name) = &arg.name {
                create_role = create_role.name(&name);
            }
            if let Some(permissions) = arg.permissions {
                let parsed: u64 = permissions.parse()?;
                create_role = create_role.permissions(Permissions::from_bits_truncate(parsed));
            }
            if let Some(unicode_emoji) = &arg.unicode_emoji {
                create_role = create_role.unicode_emoji(&unicode_emoji);
            }

            Ok(create_role.await)
        })
        .await?
        .model()
        .await?;

        Ok(created.into())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_update_role(&self, arg: OpUpdateRoleFields) -> Result<Role, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let role_id = parse_discord_id(&arg.role_id)?;

        let updated = discord_request_with_extra_error(&self.state, async move {
            let mut update_role = rt_ctx
                .discord_config
                .client
                .update_role(rt_ctx.guild_id, role_id);

            if let Some(color) = arg.color {
                update_role = update_role.color(color);
            }
            if let Some(hoist) = arg.hoist {
                update_role = update_role.hoist(hoist);
            }
            if let Some(mentionable) = arg.mentionable {
                update_role = update_role.mentionable(mentionable);
            }
            if let Some(name) = &arg.name {
                update_role = update_role.name(name.as_deref());
            }
            if let Some(permissions) = arg.permissions {
                let parsed: u64 = permissions.parse()?;
                update_role = update_role.permissions(Permissions::from_bits_truncate(parsed));
            }
            if let Some(unicode_emoji) = &arg.unicode_emoji {
                update_role = update_role.unicode_emoji(&unicode_emoji);
            }

            Ok(update_role.await)
        })
        .await?
        .model()
        .await?;

        Ok(updated.into())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_update_role_positions(
        &self,
        arg: Vec<UpdateRolePosition>,
    ) -> Result<Vec<Role>, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let positions = arg
            .into_iter()
            .map(|v| {
                Ok(RolePosition {
                    id: parse_discord_id::<RoleMarker>(&v.role_id)?,
                    position: v.position as u64,
                })
            })
            .collect::<Result<Vec<_>, AnyError>>()?;

        let out = discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .update_role_positions(rt_ctx.guild_id, &positions)
                .await
        })
        .await?
        .model()
        .await?;

        Ok(out.into_iter().map(Into::into).collect())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_delete_role(&self, arg: String) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let role_id = parse_discord_id(&arg)?;

        discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .delete_role(rt_ctx.guild_id, role_id)
                .await

            // update_role.await
        })
        .await?;

        Ok(())
    }

    async fn discord_get_emojis(&self, _arg: ()) -> Result<Vec<CustomEmoji>, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let emojis = rt_ctx.bot_state.get_guild_emojis(rt_ctx.guild_id).await?;

        Ok(emojis.into_iter().map(Into::into).collect())
    }

    async fn discord_create_emoji(&self, arg: OpCreateEmoji) -> Result<CustomEmoji, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let roles = arg
            .roles
            .map(|v| parse_str_snowflake_ids::<RoleMarker>(v.into_iter()))
            .transpose()?;

        let out = discord_request(&self.state, async move {
            let mut req =
                rt_ctx
                    .discord_config
                    .client
                    .create_emoji(rt_ctx.guild_id, &arg.name, &arg.data);

            if let Some(roles) = &roles {
                req = req.roles(roles.as_slice());
            }

            req.await
        })
        .await?
        .model()
        .await?;

        Ok(out.into())
    }

    async fn discord_edit_emoji(&self, arg: OpUpdateEmoji) -> Result<CustomEmoji, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let roles = arg
            .roles
            .map(|v| parse_str_snowflake_ids::<RoleMarker>(v.into_iter()))
            .transpose()?;

        let parsed_id = parse_str_snowflake_id(&arg.id)?;

        let out = discord_request(&self.state, async move {
            let mut req = rt_ctx
                .discord_config
                .client
                .update_emoji(rt_ctx.guild_id, parsed_id.cast());

            if let Some(name) = &arg.name {
                req = req.name(name);
            }

            if let Some(roles) = &roles {
                req = req.roles(roles.as_slice());
            }

            req.await
        })
        .await?
        .model()
        .await?;

        Ok(out.into())
    }

    async fn discord_delete_emoji(&self, arg: String) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);
        let parsed_id = parse_str_snowflake_id(&arg)?;

        discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .delete_emoji(rt_ctx.guild_id, parsed_id.cast())
                .await
        })
        .await?;

        Ok(())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_get(
        &self,
        arg: OpWebhookSpecifier,
    ) -> Result<DiscordWebhook, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id = parse_str_snowflake_id(&arg.webhook_id)?;
        let has_provided_token = arg.token.is_some();

        let webhook = discord_request(&self.state, async move {
            let mut req = rt_ctx.discord_config.client.webhook(parsed_id.cast());

            if let Some(token) = &arg.token {
                req = req.token(&token)
            }

            req.await
        })
        .await?
        .model()
        .await?;

        if !has_provided_token {
            if webhook.guild_id != Some(rt_ctx.guild_id) {
                return Err(anyhow!("This webhook does not belong to this server"));
            }
        }

        Ok(webhook.into())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_get_guild(
        &self,
        _arg: (),
    ) -> Result<Vec<DiscordWebhook>, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let webhooks = discord_request(&self.state, async move {
            rt_ctx
                .discord_config
                .client
                .guild_webhooks(rt_ctx.guild_id)
                .await
        })
        .await?
        .model()
        .await?;

        Ok(webhooks.into_iter().map(Into::into).collect())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_create(
        &self,
        arg: OpCreateWebhook,
    ) -> Result<DiscordWebhook, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let channel = parse_get_guild_channel(&self.state, &rt_ctx, &arg.channel_id).await?;

        let webhook = discord_request_with_extra_error(&self.state, async move {
            let mut req = rt_ctx
                .discord_config
                .client
                .create_webhook(channel.id, &arg.name);

            if let Some(icon) = &arg.icon {
                req = req.avatar(icon)
            }

            Ok(req.await)
        })
        .await?
        .model()
        .await?;

        Ok(webhook.into())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_edit(
        &self,
        arg: OpEditWebhook,
    ) -> Result<DiscordWebhook, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id = parse_discord_id(&arg.webhook_id)?;

        let webhook = discord_request_any(&self.state, async move {
            let webhook = rt_ctx
                .discord_config
                .client
                .webhook(parsed_id)
                .await?
                .model()
                .await?;
            if webhook.guild_id != Some(rt_ctx.guild_id) {
                return Err(anyhow!("This webhook does not belong to this server"));
            }

            let mut req = rt_ctx.discord_config.client.update_webhook(parsed_id);

            if let Some(icon) = &arg.icon {
                req = req.avatar(icon.as_deref())
            }

            if let Some(name) = &arg.name {
                req = req.name(&name)
            }

            if let Some(channel_id) = &arg.channel_id {
                let parsed_channel_id = parse_discord_id(&channel_id)?;
                req = req.channel_id(parsed_channel_id)
            }

            Ok(req.await?)
        })
        .await?
        .model()
        .await?;

        Ok(webhook.into())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_edit_with_token(
        &self,
        arg: OpEditWebhookWithToken,
    ) -> Result<DiscordWebhook, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id = parse_discord_id(&arg.webhook_id)?;

        let webhook = discord_request_any(&self.state, async move {
            let mut req = rt_ctx
                .discord_config
                .client
                .update_webhook_with_token(parsed_id, &arg.token);

            if let Some(icon) = &arg.icon {
                req = req.avatar(icon.as_deref())
            }

            if let Some(name) = &arg.name {
                req = req.name(&name)
            }

            Ok(req.await?)
        })
        .await?
        .model()
        .await?;

        Ok(webhook.into())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_delete(&self, arg: OpWebhookSpecifier) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id = parse_str_snowflake_id(&arg.webhook_id)?;

        discord_request_any(&self.state, async move {
            if arg.token.is_none() {
                // ensure the webhook is part of the current guild
                let webhook = rt_ctx
                    .discord_config
                    .client
                    .webhook(parsed_id.cast())
                    .await?
                    .model()
                    .await?;

                if webhook.guild_id != Some(rt_ctx.guild_id) {
                    return Err(anyhow!("This webhook does not belong to this server"));
                }
            }
            let mut req = rt_ctx
                .discord_config
                .client
                .delete_webhook(parsed_id.cast());

            if let Some(token) = &arg.token {
                req = req.token(&token)
            }

            Ok(req.await?)
        })
        .await?;

        Ok(())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_execute(
        &self,
        arg: OpExecuteWebhook,
    ) -> Result<Message, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id: Id<WebhookMarker> = parse_str_snowflake_id(&arg.webhook_id)?.cast();

        let attachments = convert_attachments(arg.fields.attachments.unwrap_or_default())?;

        let message = discord_request_any(&self.state, async move {
            let maybe_embeds = arg
                .fields
                .embeds
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();

            let components = arg
                .fields
                .components
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?;

            let mut mc = rt_ctx
                .discord_config
                .client
                .execute_webhook(parsed_id, &arg.token)
                .embeds(&maybe_embeds)
                .components(&components);

            if let Some(content) = &arg.fields.content {
                mc = mc.content(content)
            }

            let mentions = arg.fields.allowed_mentions.map(Into::into);
            if mentions.is_some() {
                mc = mc.allowed_mentions(mentions.as_ref());
            }

            if attachments.len() > 0 {
                mc = mc.attachments(&attachments);
            }

            if let Some(avatar_url) = &arg.avatar_url {
                mc = mc.avatar_url(avatar_url)
            }

            if let Some(username) = &arg.username {
                mc = mc.username(username)
            }

            Ok(mc.wait().await?.model().await?)
        })
        .await?;

        message.try_into()
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_message_get(
        &self,
        arg: OpWebhookMessageSpecifier,
    ) -> Result<Message, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id: Id<WebhookMarker> = parse_str_snowflake_id(&arg.webhook_id)?.cast();
        let parsed_message_id: Id<MessageMarker> = parse_str_snowflake_id(&arg.message_id)?.cast();

        let message = discord_request_any(&self.state, async move {
            Ok(rt_ctx
                .discord_config
                .client
                .webhook_message(parsed_id, &arg.token, parsed_message_id)
                .await?
                .model()
                .await?)
        })
        .await?;

        message.try_into()
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_message_delete(
        &self,
        arg: OpWebhookMessageSpecifier,
    ) -> Result<(), anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id: Id<WebhookMarker> = parse_str_snowflake_id(&arg.webhook_id)?.cast();
        let parsed_message_id: Id<MessageMarker> = parse_str_snowflake_id(&arg.message_id)?.cast();

        discord_request_any(&self.state, async move {
            Ok(rt_ctx
                .discord_config
                .client
                .delete_webhook_message(parsed_id, &arg.token, parsed_message_id)
                .await?)
        })
        .await?;

        Ok(())
    }

    #[allow(async_fn_in_trait)]
    async fn discord_webhook_message_edit(
        &self,
        arg: OpUpdateWebhookMessage,
    ) -> Result<Message, anyhow::Error> {
        let rt_ctx = get_rt_ctx(&self.state);

        let parsed_id: Id<WebhookMarker> = parse_str_snowflake_id(&arg.webhook_id)?.cast();
        let parsed_message_id: Id<MessageMarker> = parse_str_snowflake_id(&arg.message_id)?.cast();

        let attachments = convert_attachments(arg.fields.attachments.unwrap_or_default())?;

        let message = discord_request_any(&self.state, async move {
            let maybe_embeds = arg
                .fields
                .embeds
                .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

            let components = arg
                .fields
                .components
                .map(|inner| {
                    inner
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;

            let mut mc = rt_ctx
                .discord_config
                .client
                .update_webhook_message(parsed_id, &arg.token, parsed_message_id)
                .embeds(maybe_embeds.as_deref())
                .components(components.as_deref());

            if let Some(content) = &arg.fields.content {
                mc = mc.content(Some(content))
            }

            let mentions = arg.fields.allowed_mentions.map(Into::into);
            if mentions.is_some() {
                mc = mc.allowed_mentions(mentions.as_ref());
            }

            if attachments.len() > 0 {
                mc = mc.attachments(&attachments);
            }

            Ok(mc.await?.model().await?)
        })
        .await?;

        message.try_into()
    }
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_invites(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::internal::invite::Invite>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let resp = discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .guild_invites(rt_ctx.guild_id)
            .await
    })
    .await?
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

    discord_request(&state, async move {
        let mut req = rt_ctx.discord_config.client.invite(&code);
        if with_counts {
            req = req.with_counts();
        }

        if with_expiration {
            req = req.with_expiration();
        }
        req.await
    })
    .await?
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

    let code_cloned = code.clone();
    let rt_ctx_cloned = rt_ctx.clone();
    // we need to make sure this invite comes from this guild
    let invite = discord_request(&state, async move {
        rt_ctx_cloned
            .discord_config
            .client
            .invite(&code_cloned)
            .await
    })
    .await?
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
    discord_request(&state, async move {
        rt_ctx.discord_config.client.delete_invite(&code).await
    })
    .await?;

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

#[op2(async)]
pub async fn op_discord_interaction_callback(
    state: Rc<RefCell<OpState>>,
    #[serde] args: InteractionCallback,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let interaction_id: Id<InteractionMarker> = parse_discord_id(&args.interaction_id)?;

    let response_fields: twilight_model::http::interaction::InteractionResponse =
        args.data.try_into()?;

    discord_request(&state, async move {
        let client = rt_ctx.discord_config.interaction_client();
        client
            .create_response(interaction_id, &args.interaction_token, &response_fields)
            .await
    })
    .await?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_interaction_get_original_response(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
) -> Result<Message, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    discord_request(&state, async move {
        let client = rt_ctx.discord_config.interaction_client();
        client.response(&token).await
    })
    .await?
    .model()
    .await?
    .try_into()
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
        .map(|inner| {
            inner
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let attachments = convert_attachments(args.fields.attachments.unwrap_or_default())?;

    discord_request_with_extra_error(&state, async move {
        let interaction_client = rt_ctx.discord_config.interaction_client();

        let mut mc = interaction_client
            .update_response(&args.interaction_token)
            .content(args.fields.content.as_deref())
            .embeds(maybe_embeds.as_deref())
            .components(components.as_deref())
            .content(args.fields.content.as_deref())
            .attachments(&attachments);

        let mentions = args.fields.allowed_mentions.map(Into::into);
        if mentions.is_some() {
            mc = mc.allowed_mentions(mentions.as_ref());
        }

        Ok(mc.await)
    })
    .await?
    .model()
    .await?
    .try_into()
}

#[op2(async)]
pub async fn op_discord_interaction_delete_original(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    discord_request(&state, async move {
        let client = rt_ctx.discord_config.interaction_client();
        client.delete_response(&token).await
    })
    .await?;

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

    discord_request(&state, async move {
        let client = rt_ctx.discord_config.interaction_client();
        client.followup(&token, id).await
    })
    .await?
    .model()
    .await?
    .try_into()
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
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()?;

    let attachments = convert_attachments(args.fields.attachments.unwrap_or_default())?;

    discord_request_with_extra_error(&state, async move {
        let interaction_client = rt_ctx.discord_config.interaction_client();

        let mut mc = interaction_client
            .create_followup(&args.interaction_token)
            .embeds(&maybe_embeds)
            .components(&components)
            .attachments(&attachments);

        if let Some(flags) = args.flags {
            mc = mc.flags(flags.into());
        }

        if let Some(content) = &args.fields.content {
            mc = mc.content(content)
        }

        let mentions = args.fields.allowed_mentions.map(Into::into);
        if mentions.is_some() {
            mc = mc.allowed_mentions(mentions.as_ref());
        }

        Ok(mc.await)
    })
    .await?
    .model()
    .await?
    .try_into()
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
        .map(|inner| {
            inner
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let attachments = convert_attachments(args.fields.attachments.unwrap_or_default())?;

    discord_request_with_extra_error(&state, async move {
        let interaction_client = rt_ctx.discord_config.interaction_client();

        let mut mc = interaction_client
            .update_followup(&args.interaction_token, message_id)
            .content(args.fields.content.as_deref())
            .embeds(maybe_embeds.as_deref())
            .components(components.as_deref())
            .content(args.fields.content.as_deref())
            .attachments(&attachments);

        let mentions = args.fields.allowed_mentions.map(Into::into);
        if mentions.is_some() {
            mc = mc.allowed_mentions(mentions.as_ref());
        }

        Ok(mc.await)
    })
    .await?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_interaction_delete_followup_message(
    state: Rc<RefCell<OpState>>,
    #[string] token: String,
    #[serde] id: Id<MessageMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    discord_request(&state, async move {
        let client = rt_ctx.discord_config.interaction_client();
        client.delete_followup(&token, id).await
    })
    .await?;

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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .create_reaction(channel_id, message_id, &(&emoji).into())
            .await
    })
    .await?;

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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .delete_current_user_reaction(channel_id, message_id, &(&emoji).into())
            .await
    })
    .await?;

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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .delete_reaction(channel_id, message_id, &(&emoji).into(), user_id)
            .await
    })
    .await?;

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

    Ok(discord_request_with_extra_error(&state, async move {
        let emoji = (&fields.emoji).into();

        let mut req = rt_ctx
            .discord_config
            .client
            .reactions(channel_id, message_id, &emoji);

        if let Some(after_str) = &fields.after {
            req = req.after(parse_str_snowflake_id(after_str)?.cast())
        }
        if let Some(limit) = fields.limit {
            req = req.limit(limit as u16);
        }

        Ok(req.await)
    })
    .await?
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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .delete_all_reactions(channel_id, message_id)
            .await
    })
    .await?;

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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .delete_all_reaction(channel_id, message_id, &(&emoji).into())
            .await
    })
    .await?;

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
    Ok(channel.try_into()?)
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_channels(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<runtime_models::internal::channel::GuildChannel>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let channels = rt_ctx.bot_state.get_channels(rt_ctx.guild_id).await?;
    Ok(channels
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()?)
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

    Ok(discord_request_with_extra_error(&state, async move {
        let mut overwrites = Vec::new();
        let edit = rt_ctx.discord_config.client.update_channel(channel_id);

        Ok(params.apply(&mut overwrites, edit)?.await)
    })
    .await?
    .model()
    .await?
    .try_into()?)
}

#[op2(async)]
#[serde]
pub async fn op_discord_create_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] params: CreateChannel,
) -> Result<runtime_models::internal::channel::GuildChannel, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    Ok(discord_request_with_extra_error(&state, async move {
        let mut overwrites = Vec::new();
        let edit = rt_ctx
            .discord_config
            .client
            .create_guild_channel(rt_ctx.guild_id, &params.name);

        Ok(params.apply(&mut overwrites, edit)?.await)
    })
    .await?
    .model()
    .await?
    .try_into()?)
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

    Ok(discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .delete_channel(channel_id)
            .await
    })
    .await?
    .model()
    .await?
    .try_into()?)
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

    discord_request_with_extra_error(&state, async move {
        let conv = permission_overwrite
            .try_into()
            .map_err(|_| anyhow!("invalid id"))?;

        Ok(rt_ctx
            .discord_config
            .client
            .update_channel_permission(channel_id, &conv)
            .await)
    })
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

    discord_request(&state, async move {
        let req = rt_ctx
            .discord_config
            .client
            .delete_channel_permission(channel_id);

        match kind {
            PermissionOverwriteType::Member => req.member(overwrite_id.cast()).await,
            PermissionOverwriteType::Role => req.role(overwrite_id.cast()).await,
        }
    })
    .await?;

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

    let resp = discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .channel_invites(channel.id)
            .await
    })
    .await?
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

    discord_request_with_extra_error(&state, async move {
        let mut req = rt_ctx.discord_config.client.create_invite(channel.id);

        if let Some(max_age) = fields.max_age {
            req = req.max_age(max_age);
        }
        if let Some(max_uses) = fields.max_uses {
            req = req.max_uses(max_uses);
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

        Ok(req.await)
    })
    .await?
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

    let pins = discord_request(&state, async move {
        rt_ctx.discord_config.client.pins(channel_id).await
    })
    .await?
    .model()
    .await?;

    pins.into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()
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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .create_pin(channel_id, message_id)
            .await
    })
    .await?;

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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .delete_pin(channel_id, message_id)
            .await
    })
    .await?;

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

            let cloned_discord = rt_ctx.discord_config.clone();
            let guild_id = rt_ctx.guild_id;

            let resp = discord_request(state, async move {
                cloned_discord.client.guild_member(guild_id, id).await
            })
            .await;

            match resp {
                Ok(next) => {
                    let member = next.model().await?;
                    res.push(Some(member.into()))
                }
                Err(err) => {
                    let class = get_custom_error_class(&err);

                    // Handle unknown members by pushing null results
                    if !matches!(class, Some(DISCORD_NOT_FOUND_CLASS_NAME),) {
                        return Err(err);
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

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .add_guild_member_role(rt_ctx.guild_id, user_id, role_id)
            .await
    })
    .await?;

    Ok(())
}

#[op2(async)]
pub async fn op_discord_remove_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] role_id: Id<RoleMarker>,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .remove_guild_member_role(rt_ctx.guild_id, user_id, role_id)
            .await
    })
    .await?;

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

    Ok(discord_request_with_extra_error(&state, async move {
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
            builder = builder.nick(maybe_nick.as_deref())
        }

        if let Some(roles) = &fields.roles {
            builder = builder.roles(roles);
        }

        if let Some(ts) = &fields.communication_disabled_until {
            builder = builder.communication_disabled_until(
                ts.map(|v| twilight_model::util::Timestamp::from_micros(v.0 as i64 * 1000))
                    .transpose()?,
            );
        }

        Ok(builder.await)
    })
    .await?
    .model()
    .await?
    .into())
}

// Bans
#[op2(async)]
pub async fn op_discord_create_ban(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
    #[serde] extras: CreateBanFields,
) -> Result<(), AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    discord_request_with_extra_error(&state, async move {
        let mut req = rt_ctx
            .discord_config
            .client
            .create_ban(rt_ctx.guild_id, user_id);

        if let Some(days) = extras.delete_message_days {
            req = req.delete_message_seconds(days * 24 * 60 * 60);
        }

        if let Some(reason) = &extras.audit_log_reason {
            req = req.reason(reason);
        }

        Ok(req.await)
    })
    .await?;

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_ban(
    state: Rc<RefCell<OpState>>,
    #[serde] user_id: Id<UserMarker>,
) -> Result<Ban, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    Ok(discord_request(&state, async move {
        rt_ctx
            .discord_config
            .client
            .ban(rt_ctx.guild_id, user_id)
            .await
    })
    .await?
    .model()
    .await?
    .into())
}

#[op2(async)]
#[serde]
pub async fn op_discord_get_bans(state: Rc<RefCell<OpState>>) -> Result<Vec<Ban>, AnyError> {
    let rt_ctx = get_rt_ctx(&state);

    let result = discord_request(&state, async move {
        rt_ctx.discord_config.client.bans(rt_ctx.guild_id).await
    })
    .await?
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

    discord_request_with_extra_error(&state, async move {
        let mut req = rt_ctx
            .discord_config
            .client
            .delete_ban(rt_ctx.guild_id, user_id);

        if let Some(reason) = &extras.audit_log_reason {
            req = req.reason(reason);
        }

        Ok(req.await)
    })
    .await?;

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

    discord_request_with_extra_error(&state, async move {
        let mut req = rt_ctx
            .discord_config
            .client
            .remove_guild_member(rt_ctx.guild_id, user_id);

        if let Some(reason) = &extras.audit_log_reason {
            req = req.reason(reason);
        }

        Ok(req.await)
    })
    .await?;

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
        let member = discord_request_retry(&state, |config| async {
            let config = config;
            Ok(config.client.guild_member(rt_ctx.guild_id, user_id).await?)
        })
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct BlDiscordApiError {
    pub code: u64,
    pub message: String,
    pub errors: Option<serde_json::Value>,
}

impl Display for BlDiscordApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_str("Error code ")?;
        Display::fmt(&self.code, f)?;
        f.write_str(": ")?;

        f.write_str(&self.message)?;
        if let Some(errors) = &self.errors {
            f.write_str(" (")?;
            Display::fmt(errors, f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
}
