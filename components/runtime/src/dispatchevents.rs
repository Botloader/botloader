use twilight_model::{gateway::event::Event, id::GuildId};

pub fn discord_event_to_dispatch(evt: Event) -> Option<DiscordDispatchEvent> {
    match evt {
        Event::MessageCreate(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_CREATE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(&runtime_models::discord::message::Message::from(m.0))
                .unwrap(),
        }),
        Event::MessageUpdate(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_UPDATE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(
                &runtime_models::events::message_update::MessageUpdate::from(*m),
            )
            .unwrap(),
        }),
        Event::MessageDelete(m) if m.guild_id.is_some() => Some(DiscordDispatchEvent {
            name: "MESSAGE_DELETE",
            guild_id: m.guild_id.unwrap(),
            data: serde_json::to_value(
                &runtime_models::events::message_delete::MessageDelete::from(m),
            )
            .unwrap(),
        }),
        Event::MemberAdd(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_ADD",
            guild_id: m.guild_id,
            data: serde_json::to_value(&runtime_models::discord::member::Member::from(m.0))
                .unwrap(),
        }),
        Event::MemberUpdate(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_UPDATE",
            guild_id: m.guild_id,
            data: serde_json::to_value(&runtime_models::discord::member::Member::from(*m)).unwrap(),
        }),
        Event::MemberRemove(m) => Some(DiscordDispatchEvent {
            name: "MEMBER_REMOVE",
            guild_id: m.guild_id,
            data: serde_json::to_value(&runtime_models::events::member_remove::MemberRemove::from(
                m,
            ))
            .unwrap(),
        }),
        Event::InteractionCreate(interaction) => match interaction.0 {
            twilight_model::application::interaction::Interaction::Ping(_) => None,
            twilight_model::application::interaction::Interaction::MessageComponent(_) => None,
            twilight_model::application::interaction::Interaction::ApplicationCommand(cmd) => {
                let guild_id = cmd.guild_id;
                Some(DiscordDispatchEvent {
                    name: "BOTLOADER_COMMAND_INTERACTION_CREATE",
                    guild_id: guild_id.unwrap(),
                    data: serde_json::to_value(
                        &runtime_models::events::command_interaction::CommandInteraction::from(
                            *cmd,
                        ),
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
    pub guild_id: GuildId,
    pub name: &'static str,
    pub data: serde_json::Value,
}
