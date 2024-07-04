use super::{member::Member, messages::Message, script::CommandOptionType};
use crate::{
    discord::{component::ComponentType, message::Attachment},
    util::NotBigI64,
};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "bindings/internal/Interaction.ts")]
pub enum Interaction {
    Command(Box<CommandInteraction>),
    MessageComponent(MessageComponentInteraction),
    ModalSubmit(ModalInteraction),
}

impl TryFrom<twilight_model::application::interaction::Interaction> for Interaction {
    type Error = anyhow::Error;

    fn try_from(
        v: twilight_model::application::interaction::Interaction,
    ) -> Result<Self, Self::Error> {
        match v.data {
            Some(
                twilight_model::application::interaction::InteractionData::ApplicationCommand(data),
            ) => {
                let mut name = data.name;
                let mut parent_name: Option<String> = None;
                let mut parent_parent_name: Option<String> = None;
                let mut opts: Vec<CommandInteractionOption> = Vec::new();

                for opt in data.options {
                    match opt.value {
                        CommandOptionValue::SubCommand(sub_cmd) => {
                            // fix names, original name was the parent group name
                            let old = std::mem::replace(&mut name, opt.name.clone());
                            parent_name = Some(old);

                            for sub_opt in sub_cmd {
                                opts.push(sub_opt.into());
                            }
                        }
                        CommandOptionValue::SubCommandGroup(sub_cmd_group) => {
                            // fix names, original name was the parent of the parent group name
                            parent_name = Some(opt.name.clone());
                            parent_parent_name = Some(name.clone());

                            // there can only be 1, maybe add a check for that at some point
                            // never know when discord might break stuff
                            let cmd = sub_cmd_group[0].clone();
                            name = cmd.name;
                            if let CommandOptionValue::SubCommand(sub_sub_opts) = cmd.value {
                                for sub_sub_opt in sub_sub_opts {
                                    opts.push(sub_sub_opt.into());
                                }
                            }
                        }
                        _ => {
                            opts.push(opt.into());
                        }
                    }
                }

                let is_autocomplete = matches!(v.kind, twilight_model::application::interaction::InteractionType::ApplicationCommandAutocomplete);

                Ok(Self::Command(Box::new(CommandInteraction {
                    name,
                    parent_name,
                    parent_parent_name,
                    options: opts,
                    channel_id: v.channel.unwrap().id.to_string(),
                    id: v.id.to_string(),
                    member: Member::from_partial(v.member.unwrap()),
                    token: v.token,
                    data_map: data
                        .resolved
                        .map(TryInto::try_into)
                        .transpose()?
                        .unwrap_or_default(),

                    kind: data.kind.into(),
                    target_id: data.target_id.as_ref().map(ToString::to_string),
                    is_autocomplete,
                })))
            }
            Some(twilight_model::application::interaction::InteractionData::MessageComponent(
                data,
            )) => Ok(Self::MessageComponent(MessageComponentInteraction {
                channel_id: v.channel.unwrap().id.to_string(),
                guild_locale: v.guild_locale,
                id: v.id.to_string(),
                locale: v.locale.unwrap_or_default(),
                member: Member::from_partial(v.member.unwrap()),
                message: v.message.unwrap().try_into()?,
                token: v.token,
                custom_id: data.custom_id,
                component_type: data.component_type.into(),
                values: data.values,
                resolved: data.resolved.map(|rv| rv.try_into()).transpose()?,
            })),
            Some(twilight_model::application::interaction::InteractionData::ModalSubmit(data)) => {
                Ok(Self::ModalSubmit(ModalInteraction {
                    channel_id: v.channel.unwrap().id.to_string(),
                    guild_locale: v.guild_locale,
                    id: v.id.to_string(),
                    locale: v.locale.unwrap_or_default(),
                    member: Member::from_partial(v.member.unwrap()),
                    message: v.message.map(TryInto::try_into).transpose()?,
                    token: v.token,
                    custom_id: data.custom_id,
                    values: data
                        .components
                        .into_iter()
                        .flat_map(|row| {
                            row.components
                                .into_iter()
                                .map(ModalInteractionDataComponent::from)
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                }))
            }
            Some(_) => Err(anyhow::anyhow!(
                "unknown interaction data for interaction type {}",
                v.kind.kind()
            )),
            None => Err(anyhow::anyhow!(
                "no interaction data for interaction type {}",
                v.kind.kind()
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "bindings/internal/MessageComponentInteraction.ts")]
#[serde(rename_all = "camelCase")]
pub struct MessageComponentInteraction {
    pub channel_id: String,
    pub guild_locale: Option<String>,
    pub id: String,
    pub locale: String,
    pub member: Member,
    pub message: Message,
    pub token: String,
    pub resolved: Option<InteractionDataMap>,

    pub custom_id: String,
    pub component_type: ComponentType,
    pub values: Vec<String>,
}

use std::collections::HashMap;

use serde::Deserialize;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::InteractionDataResolved;

use crate::{
    discord::role::Role,
    internal::{
        interactions::{InteractionPartialChannel, InteractionPartialMember},
        user::User,
    },
};

// we perform some normalization to make things simpler on the script side
// and also simpler overall
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CommandInteraction.ts")]
#[serde(rename_all = "camelCase")]
pub struct CommandInteraction {
    pub channel_id: String,

    pub id: String,
    pub member: Member,
    pub token: String,

    pub name: String,
    pub parent_name: Option<String>,
    pub parent_parent_name: Option<String>,

    pub options: Vec<CommandInteractionOption>,
    pub data_map: InteractionDataMap,

    pub kind: CommandType,
    pub target_id: Option<String>,

    pub is_autocomplete: bool,
}

#[derive(Clone, Debug, Serialize, TS, Default)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionDataMaps.ts")]
#[serde(rename_all = "camelCase")]
pub struct InteractionDataMap {
    pub channels: HashMap<String, InteractionPartialChannel>,
    pub members: HashMap<String, InteractionPartialMember>,
    pub messages: HashMap<String, Message>,
    pub roles: HashMap<String, Role>,
    pub users: HashMap<String, User>,
    pub attachments: HashMap<String, Attachment>,
}

impl TryFrom<InteractionDataResolved> for InteractionDataMap {
    type Error = anyhow::Error;

    fn try_from(v: InteractionDataResolved) -> Result<Self, Self::Error> {
        Ok(Self {
            channels: v
                .channels
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
            members: v
                .members
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
            messages: v
                .messages
                .into_iter()
                .map(|(k, v)| Ok((k.to_string(), v.try_into()?)))
                .collect::<Result<_, anyhow::Error>>()?,
            roles: v
                .roles
                .into_iter()
                .map(|(k, v)| (k.to_string(), (&v).into()))
                .collect(),
            users: v
                .users
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
            attachments: v
                .attachments
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CommandInteractionOption.ts")]
#[serde(rename_all = "camelCase")]
pub struct CommandInteractionOption {
    pub name: String,
    pub value: CommandInteractionOptionValue,
}

impl From<CommandDataOption> for CommandInteractionOption {
    fn from(v: CommandDataOption) -> Self {
        Self {
            name: v.name,
            value: v.value.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CommandInteractionOptionValue.ts")]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CommandInteractionOptionValue {
    String {
        value: String,
    },
    Integer {
        value: NotBigI64,
    },
    Boolean {
        value: bool,
    },
    User {
        value: String,
    },
    Channel {
        value: String,
    },
    Role {
        value: String,
    },
    Mentionable {
        value: String,
    },
    Number {
        value: f64,
    },
    Attachment {
        value: String,
    },
    Focused {
        value: String,
        option_kind: CommandOptionType,
    },
}

impl From<CommandOptionValue> for CommandInteractionOptionValue {
    fn from(v: CommandOptionValue) -> Self {
        match v {
            CommandOptionValue::String(ov) => Self::String { value: ov },
            CommandOptionValue::Integer(ov) => Self::Integer {
                value: NotBigI64(ov),
            },
            CommandOptionValue::Boolean(ov) => Self::Boolean { value: ov },
            CommandOptionValue::User(ov) => Self::User {
                value: ov.to_string(),
            },
            CommandOptionValue::Channel(ov) => Self::Channel {
                value: ov.to_string(),
            },
            CommandOptionValue::Role(ov) => Self::Role {
                value: ov.to_string(),
            },
            CommandOptionValue::Mentionable(ov) => Self::Mentionable {
                value: ov.to_string(),
            },
            CommandOptionValue::Number(ov) => Self::Number { value: ov },
            // the below are unreachable because of previous checkcs
            // altough it might be a bad idea since this is more or less a public function
            // aaa i should change it yeah.... later
            CommandOptionValue::SubCommand(_) => unreachable!(),
            CommandOptionValue::SubCommandGroup(_) => unreachable!(),
            CommandOptionValue::Attachment(v) => Self::Attachment {
                value: v.to_string(),
            },
            CommandOptionValue::Focused(ov, k) => Self::Focused {
                value: ov,
                option_kind: match k {
                    twilight_model::application::command::CommandOptionType::String => {
                        CommandOptionType::String
                    }
                    twilight_model::application::command::CommandOptionType::Integer => {
                        CommandOptionType::Integer
                    }
                    twilight_model::application::command::CommandOptionType::Boolean => {
                        CommandOptionType::Boolean
                    }
                    twilight_model::application::command::CommandOptionType::User => {
                        CommandOptionType::User
                    }
                    twilight_model::application::command::CommandOptionType::Channel => {
                        CommandOptionType::Channel
                    }
                    twilight_model::application::command::CommandOptionType::Role => {
                        CommandOptionType::Role
                    }
                    twilight_model::application::command::CommandOptionType::Mentionable => {
                        CommandOptionType::Mentionable
                    }
                    twilight_model::application::command::CommandOptionType::Number => {
                        CommandOptionType::Number
                    }
                    twilight_model::application::command::CommandOptionType::SubCommand => {
                        unreachable!()
                    }
                    twilight_model::application::command::CommandOptionType::SubCommandGroup => {
                        unreachable!()
                    }
                    twilight_model::application::command::CommandOptionType::Attachment => {
                        CommandOptionType::Attachment
                    }
                    _ => todo!(),
                },
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CommandType.ts")]
pub enum CommandType {
    Chat,
    User,
    Message,
}

impl From<twilight_model::application::command::CommandType> for CommandType {
    fn from(v: twilight_model::application::command::CommandType) -> Self {
        match v {
            twilight_model::application::command::CommandType::ChatInput => Self::Chat,
            twilight_model::application::command::CommandType::User => Self::User,
            twilight_model::application::command::CommandType::Message => Self::Message,
            _ => todo!(),
        }
    }
}

impl From<CommandType> for twilight_model::application::command::CommandType {
    fn from(v: CommandType) -> Self {
        match v {
            CommandType::Chat => Self::ChatInput,
            CommandType::User => Self::User,
            CommandType::Message => Self::Message,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteraction.ts",
    rename = "IModalInteraction"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteraction {
    pub channel_id: String,
    pub guild_locale: Option<String>,
    pub id: String,
    pub locale: String,
    pub member: Member,
    pub message: Option<Message>,
    pub token: String,

    pub custom_id: String,
    pub values: Vec<ModalInteractionDataComponent>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionDataComponent.ts",
    rename = "IModalInteractionDataComponent"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionDataComponent {
    pub custom_id: String,
    pub kind: ComponentType,
    pub value: String,
}

use twilight_model::application::interaction::modal::ModalInteractionDataComponent as TwilightModalInteractionDataComponent;
impl From<TwilightModalInteractionDataComponent> for ModalInteractionDataComponent {
    fn from(v: TwilightModalInteractionDataComponent) -> Self {
        Self {
            custom_id: v.custom_id,
            kind: v.kind.into(),
            value: v.value.unwrap_or_default(),
        }
    }
}
