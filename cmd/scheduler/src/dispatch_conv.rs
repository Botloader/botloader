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
            data: serde_json::to_value(
                &runtime_models::internal::events::EventMessageUpdate::from(*m),
            )
            .unwrap(),
        }),
        DispatchEvent::MessageDelete(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_DELETE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::discord::events::EventMessageDelete::from(
                m,
            ))
            .unwrap(),
        }),
        DispatchEvent::MemberAdd(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_ADD",
            guild_id: m.guild_id,
            data: serde_json::to_value(&runtime_models::internal::member::Member::from(m.0))
                .unwrap(),
        }),
        DispatchEvent::MemberUpdate(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_UPDATE",
            guild_id: m.guild_id,
            data: serde_json::to_value(&runtime_models::internal::member::Member::from(*m))
                .unwrap(),
        }),
        DispatchEvent::MemberRemove(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_REMOVE",
            guild_id: m.guild_id,
            data: serde_json::to_value(&runtime_models::internal::events::EventMemberRemove::from(
                m,
            ))
            .unwrap(),
        }),
        DispatchEvent::ReactionAdd(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_ADD",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                &runtime_models::internal::events::EventMessageReactionAdd::from(*r),
            )
            .unwrap(),
        }),
        DispatchEvent::ReactionRemove(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                &runtime_models::discord::events::EventMessageReactionRemove::from(*r),
            )
            .unwrap(),
        }),
        DispatchEvent::ReactionRemoveAll(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE_ALL",
            guild_id: r.guild_id.expect("only guild event sent to guild worker"),
            data: serde_json::to_value(
                &runtime_models::discord::events::EventMessageReactionRemoveAll::from(r),
            )
            .unwrap(),
        }),
        DispatchEvent::ReactionRemoveEmoji(r) => Some(DiscordDispatchEvent {
            name: "MESSAGE_REACTION_REMOVE_ALL_EMOJI",
            guild_id: r.guild_id,
            data: serde_json::to_value(
                &runtime_models::discord::events::EventMessageReactionRemoveAllEmoji::from(r),
            )
            .unwrap(),
        }),
        DispatchEvent::InteractionCreate(interaction) => match interaction.0 {
            twilight_model::application::interaction::Interaction::Ping(_) => None,
            twilight_model::application::interaction::Interaction::MessageComponent(comp) => {
                let guild_id = comp.guild_id;
                Some(DiscordDispatchEvent {
                    name: "BOTLOADER_COMPONENT_INTERACTION_CREATE",
                    guild_id: guild_id.unwrap(),
                    data: serde_json::to_value(
                        &runtime_models::internal::component_interaction::MessageComponentInteraction::from(
                            *comp,
                        ),
                    )
                    .unwrap(),
                })
            }
            twilight_model::application::interaction::Interaction::ApplicationCommand(cmd) => {
                let guild_id = cmd.guild_id;
                Some(DiscordDispatchEvent {
                    name: "BOTLOADER_COMMAND_INTERACTION_CREATE",
                    guild_id: guild_id.unwrap(),
                    data: serde_json::to_value(
                        &runtime_models::internal::command_interaction::CommandInteraction::from(
                            *cmd,
                        ),
                    )
                    .unwrap(),
                })
            }
            twilight_model::application::interaction::Interaction::ModalSubmit(m) => {
                let guild_id = m.guild_id;
                Some(DiscordDispatchEvent {
                    name: "BOTLOADER_MODAL_SUBMIT_INTERACTION_CREATE",
                    guild_id: guild_id.unwrap(),
                    data: serde_json::to_value(
                        &runtime_models::internal::modal_interaction::ModalInteraction::from(*m),
                    )
                    .unwrap(),
                })
            }
            _ => None,
        },
        _ => None,
    }
}

pub struct DiscordDispatchEvent {
    pub guild_id: Id<GuildMarker>,
    pub name: &'static str,
    pub data: serde_json::Value,
}
