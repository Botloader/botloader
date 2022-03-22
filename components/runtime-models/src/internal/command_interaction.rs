use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::application::interaction::{
    application_command::{CommandDataOption, CommandInteractionDataResolved, CommandOptionValue},
    ApplicationCommand,
};

use crate::{
    discord::role::Role,
    internal::{
        interactions::{InteractionPartialChannel, InteractionPartialMember},
        member::Member,
        messages::Message,
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
    pub data_map: CommandInteractionDataMap,

    pub kind: CommandType,
    pub target_id: Option<String>,
}

impl From<ApplicationCommand> for CommandInteraction {
    fn from(cmd: ApplicationCommand) -> Self {
        let mut name = cmd.data.name;
        let mut parent_name: Option<String> = None;
        let mut parent_parent_name: Option<String> = None;
        let mut opts: Vec<CommandInteractionOption> = Vec::new();

        for opt in cmd.data.options {
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

        Self {
            name,
            parent_name,
            parent_parent_name,
            options: opts,
            channel_id: cmd.channel_id.to_string(),
            id: cmd.id.to_string(),
            member: Member::from_partial(cmd.member.unwrap()),
            token: cmd.token,
            data_map: cmd.data.resolved.map(Into::into).unwrap_or_default(),

            kind: cmd.data.kind.into(),
            target_id: cmd.data.target_id.as_ref().map(ToString::to_string),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS, Default)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CommandInteractionDataMaps.ts")]
#[serde(rename_all = "camelCase")]
pub struct CommandInteractionDataMap {
    pub channels: HashMap<String, InteractionPartialChannel>,
    pub members: HashMap<String, InteractionPartialMember>,
    pub messages: HashMap<String, Message>,
    pub roles: HashMap<String, Role>,
    pub users: HashMap<String, User>,
}

impl From<CommandInteractionDataResolved> for CommandInteractionDataMap {
    fn from(v: CommandInteractionDataResolved) -> Self {
        Self {
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
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
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
        }
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
    String { value: String },
    Integer { value: i64 },
    Boolean { value: bool },
    User { value: String },
    Channel { value: String },
    Role { value: String },
    Mentionable { value: String },
    Number { value: f64 },
}

impl From<CommandOptionValue> for CommandInteractionOptionValue {
    fn from(v: CommandOptionValue) -> Self {
        match v {
            CommandOptionValue::String(ov) => Self::String { value: ov },
            CommandOptionValue::Integer(ov) => Self::Integer { value: ov },
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
            CommandOptionValue::Number(ov) => Self::Number { value: ov.0 },
            // the below are unreachable because of previous checkcs
            // altough it might be a bad idea since this is more or less a public function
            // aaa i should change it yeah.... later
            CommandOptionValue::SubCommand(_) => unreachable!(),
            CommandOptionValue::SubCommandGroup(_) => unreachable!(),
            CommandOptionValue::Attachment(_) => Self::String {
                value: "TODO".to_string(),
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
