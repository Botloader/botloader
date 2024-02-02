use twilight_model::{
    gateway::event::DispatchEvent,
    id::{marker::GuildMarker, Id},
};

pub fn discord_event_to_dispatch(evt: DispatchEvent) -> Option<DiscordDispatchEvent> {
    match evt {
        DispatchEvent::MessageCreate(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_CREATE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::internal::messages::Message::from(m.0))
                .unwrap(),
        }),
        DispatchEvent::MessageUpdate(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_UPDATE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(runtime_models::internal::events::EventMessageUpdate::from(
                *m,
            ))
            .unwrap(),
        }),
        DispatchEvent::MessageDelete(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_DELETE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(runtime_models::discord::events::EventMessageDelete::from(
                m,
            ))
            .unwrap(),
        }),
        DispatchEvent::MemberAdd(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_ADD",
            guild_id: m.guild_id,
            data: serde_json::to_value(runtime_models::internal::member::Member::from(m.member))
                .unwrap(),
        }),
        DispatchEvent::MemberUpdate(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_UPDATE",
            guild_id: m.guild_id,
            data: serde_json::to_value(runtime_models::internal::member::Member::from(*m)).unwrap(),
        }),
        DispatchEvent::MemberRemove(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_REMOVE",
            guild_id: m.guild_id,
            data: serde_json::to_value(runtime_models::internal::events::EventMemberRemove::from(
                m,
            ))
            .unwrap(),
        }),
        DispatchEvent::ReactionAdd(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_ADD",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                runtime_models::internal::events::EventMessageReactionAdd::from(*r),
            )
            .unwrap(),
        }),
        DispatchEvent::ReactionRemove(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                runtime_models::discord::events::EventMessageReactionRemove::from(*r),
            )
            .unwrap(),
        }),
        DispatchEvent::ReactionRemoveAll(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE_ALL",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                runtime_models::discord::events::EventMessageReactionRemoveAll::from(r),
            )
            .unwrap(),
        }),
        DispatchEvent::ReactionRemoveEmoji(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE_ALL_EMOJI",
            guild_id: r.guild_id,
            data: serde_json::to_value(
                runtime_models::discord::events::EventMessageReactionRemoveAllEmoji::from(r),
            )
            .unwrap(),
        }),
        DispatchEvent::ChannelCreate(cc) => Some(DiscordDispatchEvent {
            name: "CHANNEL_CREATE",
            guild_id: cc.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::from(
                cc.0,
            ))
            .unwrap(),
        }),
        DispatchEvent::ChannelUpdate(cu) => Some(DiscordDispatchEvent {
            name: "CHANNEL_UPDATE",
            guild_id: cu.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::from(
                cu.0,
            ))
            .unwrap(),
        }),
        DispatchEvent::ChannelDelete(cd) => Some(DiscordDispatchEvent {
            name: "CHANNEL_DELETE",
            guild_id: cd.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::from(
                cd.0,
            ))
            .unwrap(),
        }),
        DispatchEvent::ThreadCreate(r) => Some(DiscordDispatchEvent {
            name: "THREAD_CREATE",
            guild_id: r.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::from(r.0))
                .unwrap(),
        }),
        DispatchEvent::ThreadUpdate(r) => Some(DiscordDispatchEvent {
            name: "THREAD_UPDATE",
            guild_id: r.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::internal::channel::GuildChannel::from(r.0))
                .unwrap(),
        }),
        DispatchEvent::ThreadDelete(r) => Some(DiscordDispatchEvent {
            name: "THREAD_DELETE",
            guild_id: r.guild_id,
            data: serde_json::to_value(runtime_models::discord::events::EventThreadDelete::from(r))
                .unwrap(),
        }),
        DispatchEvent::InteractionCreate(interaction) => {
            let guild_id = interaction.guild_id.unwrap();

            if let Ok(v) =
                runtime_models::internal::interaction::Interaction::try_from(interaction.0)
            {
                match v {
                    runtime_models::internal::interaction::Interaction::Command(
                        cmd_interaction,
                    ) => Some(DiscordDispatchEvent {
                        guild_id,
                        name: "BOTLOADER_COMMAND_INTERACTION_CREATE",
                        data: serde_json::to_value(&cmd_interaction).unwrap(),
                    }),
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
            } else {
                None
            }
        }
        DispatchEvent::InviteCreate(invite) => Some(DiscordDispatchEvent {
            guild_id: invite.guild_id,
            name: "INVITE_CREATE",
            data: serde_json::to_value(
                runtime_models::internal::events::EventInviteCreate::try_from(*invite)
                    .map_err(|err| {
                        tracing::error!(
                            "failed converting dispatch event InviteCreate event: {err:?}"
                        );
                    })
                    .ok(),
            )
            .unwrap(),
        }),
        DispatchEvent::InviteDelete(invite) => Some(DiscordDispatchEvent {
            guild_id: invite.guild_id,
            name: "INVITE_DELETE",
            data: serde_json::to_value(runtime_models::internal::events::EventInviteDelete::from(
                invite,
            ))
            .unwrap(),
        }),

        _ => None,
    }
}

pub struct DiscordDispatchEvent {
    pub guild_id: Id<GuildMarker>,
    pub name: &'static str,
    pub data: serde_json::Value,
}
