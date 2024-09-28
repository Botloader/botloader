use dbrokerapi::broker_scheduler_rpc::{DiscordEvent, DiscordEventData};
use runtime_models::internal::events::VoiceState;
use twilight_model::id::{marker::GuildMarker, Id};

pub fn discord_event_to_dispatch(
    evt: DiscordEvent,
) -> Result<Option<DiscordDispatchEvent>, anyhow::Error> {
    Ok(match evt.event {
        // Messages
        DiscordEventData::MessageCreate(m) => Some(DiscordDispatchEvent {
            name: "MESSAGE_CREATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::internal::messages::Message::try_from(
                m.0,
            )?)
            .unwrap(),
        }),
        DiscordEventData::MessageUpdate(m) => Some(DiscordDispatchEvent {
            name: "MESSAGE_UPDATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(
                runtime_models::internal::events::EventMessageUpdate::try_from(*m)?,
            )
            .unwrap(),
        }),
        DiscordEventData::MessageDelete(m) => Some(DiscordDispatchEvent {
            name: "MESSAGE_DELETE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(runtime_models::discord::events::EventMessageDelete::from(
                m,
            ))
            .unwrap(),
        }),

        // Members
        DiscordEventData::MemberAdd(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_ADD",
            guild_id: m.guild_id,
            data: serde_json::to_value(runtime_models::internal::member::Member::from(m.member))
                .unwrap(),
        }),
        DiscordEventData::MemberUpdate(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_UPDATE",
            guild_id: m.guild_id,
            data: serde_json::to_value(runtime_models::internal::member::Member::from(*m)).unwrap(),
        }),
        DiscordEventData::MemberRemove(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_REMOVE",
            guild_id: m.guild_id,
            data: serde_json::to_value(runtime_models::internal::events::EventMemberRemove::from(
                m,
            ))
            .unwrap(),
        }),

        // Reactions
        DiscordEventData::ReactionAdd(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_ADD",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                runtime_models::internal::events::EventMessageReactionAdd::from(*r),
            )
            .unwrap(),
        }),
        DiscordEventData::ReactionRemove(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                runtime_models::discord::events::EventMessageReactionRemove::from(*r),
            )
            .unwrap(),
        }),
        DiscordEventData::ReactionRemoveAll(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE_ALL",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                runtime_models::discord::events::EventMessageReactionRemoveAll::from(r),
            )
            .unwrap(),
        }),
        DiscordEventData::ReactionRemoveEmoji(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE_ALL_EMOJI",
            guild_id: r.guild_id,
            data: serde_json::to_value(
                runtime_models::discord::events::EventMessageReactionRemoveAllEmoji::from(r),
            )
            .unwrap(),
        }),

        // Channels
        DiscordEventData::ChannelCreate(cc) => Some(DiscordDispatchEvent {
            name: "CHANNEL_CREATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::try_from(
                cc.0,
            )?)
            .unwrap(),
        }),
        DiscordEventData::ChannelUpdate(cu) => Some(DiscordDispatchEvent {
            name: "CHANNEL_UPDATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::try_from(
                cu.0,
            )?)
            .unwrap(),
        }),
        DiscordEventData::ChannelDelete(cd) => Some(DiscordDispatchEvent {
            name: "CHANNEL_DELETE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::try_from(
                cd.0,
            )?)
            .unwrap(),
        }),

        // Threads
        DiscordEventData::ThreadCreate(r) => Some(DiscordDispatchEvent {
            name: "THREAD_CREATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::try_from(
                r.0,
            )?)
            .unwrap(),
        }),
        DiscordEventData::ThreadUpdate(r) => Some(DiscordDispatchEvent {
            name: "THREAD_UPDATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::try_from(
                r.0,
            )?)
            .unwrap(),
        }),
        DiscordEventData::ThreadDelete(r) => Some(DiscordDispatchEvent {
            name: "THREAD_DELETE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(
                runtime_models::discord::events::EventThreadDelete::try_from(r)?,
            )
            .unwrap(),
        }),
        DiscordEventData::ThreadListSync(r) => Some(DiscordDispatchEvent {
            name: "THREAD_LIST_SYNC",
            guild_id: evt.guild_id,
            data: serde_json::to_value(
                runtime_models::internal::events::EventThreadListSync::try_from(r)?,
            )
            .unwrap(),
        }),
        DiscordEventData::ThreadMemberUpdate(r) => Some(DiscordDispatchEvent {
            name: "THREAD_MEMBER_UPDATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(runtime_models::internal::channel::ThreadMember::from(
                r.member,
            ))
            .unwrap(),
        }),
        DiscordEventData::ThreadMembersUpdate(r) => Some(DiscordDispatchEvent {
            name: "THREAD_MEMBERS_UPDATE",
            guild_id: r.guild_id,
            data: serde_json::to_value(
                runtime_models::internal::events::EventThreadMembersUpdate::from(r),
            )
            .unwrap(),
        }),

        // Roles
        DiscordEventData::RoleCreate(v) => Some(DiscordDispatchEvent {
            name: "ROLE_CREATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::discord::role::Role::from(v.role)).unwrap(),
        }),
        DiscordEventData::RoleUpdate(v) => Some(DiscordDispatchEvent {
            name: "ROLE_UPDATE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::discord::role::Role::from(v.role)).unwrap(),
        }),
        DiscordEventData::RoleDelete(v) => Some(DiscordDispatchEvent {
            name: "ROLE_DELETE",
            guild_id: evt.guild_id,
            data: serde_json::to_value(&runtime_models::discord::events::EventRoleDelete::from(v))
                .unwrap(),
        }),

        // Interactions
        DiscordEventData::InteractionCreate(interaction) => {
            let guild_id = evt.guild_id;

            let v = runtime_models::internal::interaction::Interaction::try_from(interaction.0)?;
            match v {
                runtime_models::internal::interaction::Interaction::Command(cmd_interaction) => {
                    Some(DiscordDispatchEvent {
                        guild_id,
                        name: "BOTLOADER_COMMAND_INTERACTION_CREATE",
                        data: serde_json::to_value(&cmd_interaction).unwrap(),
                    })
                }
                runtime_models::internal::interaction::Interaction::MessageComponent(
                    component_interaction,
                ) => Some(DiscordDispatchEvent {
                    guild_id,
                    name: "BOTLOADER_COMPONENT_INTERACTION_CREATE",
                    data: serde_json::to_value(&component_interaction).unwrap(),
                }),
                runtime_models::internal::interaction::Interaction::ModalSubmit(
                    modal_interaction,
                ) => Some(DiscordDispatchEvent {
                    guild_id,
                    name: "BOTLOADER_MODAL_SUBMIT_INTERACTION_CREATE",
                    data: serde_json::to_value(&modal_interaction).unwrap(),
                }),
            }
        }

        // Invites
        DiscordEventData::InviteCreate(invite) => Some(DiscordDispatchEvent {
            guild_id: invite.guild_id,
            name: "INVITE_CREATE",
            data: serde_json::to_value(
                runtime_models::internal::events::EventInviteCreate::try_from(*invite)?,
            )
            .unwrap(),
        }),
        DiscordEventData::InviteDelete(invite) => Some(DiscordDispatchEvent {
            guild_id: invite.guild_id,
            name: "INVITE_DELETE",
            data: serde_json::to_value(runtime_models::internal::events::EventInviteDelete::from(
                invite,
            ))
            .unwrap(),
        }),

        // Voice states
        DiscordEventData::VoiceStateUpdate { event, old_state } => Some(DiscordDispatchEvent {
            guild_id: evt.guild_id,
            name: "VOICE_STATE_UPDATE",
            data: serde_json::to_value(runtime_models::internal::events::EventVoiceStateUpdate {
                new: VoiceState::try_from(event.0)?,
                old: old_state.map(|v| VoiceState::try_from(*v)).transpose()?,
            })
            .unwrap(),
        }),
        DiscordEventData::WebhooksUpdate(wu) => Some(DiscordDispatchEvent {
            guild_id: evt.guild_id,
            name: "WEBHOOKS_UPDATE",
            data: serde_json::to_value(wu.channel_id.to_string()).unwrap(),
        }),
        DiscordEventData::GuildDelete(_) => None,
        DiscordEventData::GuildCreate(_) => None,
        DiscordEventData::MessageDeleteBulk(_) => None,
    })
}

pub struct DiscordDispatchEvent {
    pub guild_id: Id<GuildMarker>,
    pub name: &'static str,
    pub data: serde_json::Value,
}
