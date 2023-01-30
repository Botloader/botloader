use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{discord::channel::ChannelType, util::NotBigU64};

use super::interaction::CommandType;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/ScriptMeta.ts")]
pub struct ScriptMeta {
    pub description: String,
    #[ts(type = "number")]
    pub script_id: NotBigU64,
    pub commands: Vec<Command>,
    pub command_groups: Vec<CommandGroup>,
    pub interval_timers: Vec<IntervalTimer>,
    pub task_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/IntervalTimer.ts")]
pub struct IntervalTimer {
    pub name: String,
    pub interval: IntervalType,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/IntervalType.ts")]
pub enum IntervalType {
    Minutes(NotBigU64),
    Cron(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/CommandGroup.ts")]
pub struct CommandGroup {
    pub name: String,
    pub description: String,
    pub sub_groups: Vec<CommandSubGroup>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/CommandSubGroup.ts")]
pub struct CommandSubGroup {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/Command.ts")]
pub struct Command {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
    #[serde(default)]
    #[ts(optional)]
    pub group: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub sub_group: Option<String>,

    pub kind: CommandType,
}

impl From<Command> for twilight_model::application::command::CommandOption {
    fn from(cmd: Command) -> Self {
        twilight_model::application::command::CommandOption {
            name: cmd.name,
            description: cmd.description,
            options: if cmd.options.is_empty() {
                None
            } else {
                Some(cmd.options.into_iter().map(Into::into).collect())
            },
            description_localizations: Default::default(),
            name_localizations: Default::default(),
            autocomplete: None,
            channel_types: None,
            choices: None,
            kind: twilight_model::application::command::CommandOptionType::SubCommand,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            required: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/CommandOptionType.ts")]
pub enum CommandOptionType {
    // SubCommand,
    // SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
}

impl From<CommandOptionType> for twilight_model::application::command::CommandOptionType {
    fn from(v: CommandOptionType) -> Self {
        match v {
            CommandOptionType::String => Self::String,
            CommandOptionType::Integer => Self::Integer,
            CommandOptionType::Boolean => Self::Boolean,
            CommandOptionType::User => Self::User,
            CommandOptionType::Channel => Self::Channel,
            CommandOptionType::Role => Self::Role,
            CommandOptionType::Mentionable => Self::Mentionable,
            CommandOptionType::Number => Self::Number,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/CommandOption.ts")]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub kind: CommandOptionType,
    pub required: bool,
    pub extra_options: ExtraCommandOptions,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/internal/ExtraCommandOptions.ts")]
pub struct ExtraCommandOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub min_value: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_value: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub channel_types: Option<Vec<ChannelType>>,
    // #[serde(default)]
    // pub choices: Vec<OptionChoice>,
}

// #[derive(Clone, Debug, Deserialize, Serialize, TS)]
// #[ts(export)]
// #[serde(rename_all = "camelCase")]
// #[ts(export_to = "bindings/internal/CommandOptionChoice.ts")]
// pub struct OptionChoice {
//     name: String,
//     value: String,
// }

// pub enum ChoiceValue{
//     String()
// }

// impl From<OptionChoice> for twilight_model::application::command::CommandOptionChoice {
//     fn from(v: OptionChoice) -> Self {
//         Self {

//         }
//     }
// }

impl From<CommandOption> for twilight_model::application::command::CommandOption {
    fn from(v: CommandOption) -> Self {
        // use twilight_model::application::command::BaseCommandOptionData;
        // use twilight_model::application::command::ChannelCommandOptionData;
        // use twilight_model::application::command::ChoiceCommandOptionData;
        use twilight_model::application::command::CommandOptionValue;
        // use twilight_model::application::command::Number;

        match v.kind {
            CommandOptionType::String => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::String,
                autocomplete: None,
                channel_types: None,
                choices: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Integer => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::Integer,
                min_value: v
                    .extra_options
                    .min_value
                    .map(|v| CommandOptionValue::Integer(v as i64)),
                max_value: v
                    .extra_options
                    .max_value
                    .map(|v| CommandOptionValue::Integer(v as i64)),
                autocomplete: None,
                channel_types: None,
                choices: None,
                description_localizations: None,
                max_length: None,
                min_length: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Boolean => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::Boolean,
                description_localizations: Default::default(),
                name_localizations: Default::default(),
                autocomplete: None,
                channel_types: None,
                choices: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                options: None,
            },
            CommandOptionType::User => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::User,
                description_localizations: Default::default(),
                name_localizations: Default::default(),
                autocomplete: None,
                channel_types: None,
                choices: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                options: None,
            },
            CommandOptionType::Channel => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::Channel,
                channel_types: Some(
                    v.extra_options
                        .channel_types
                        .unwrap_or_default()
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                ),
                description_localizations: Default::default(),
                name_localizations: Default::default(),
                autocomplete: None,
                choices: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                options: None,
            },
            CommandOptionType::Role => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::Role,

                description_localizations: Default::default(),
                name_localizations: Default::default(),
                autocomplete: None,
                channel_types: None,
                choices: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                options: None,
            },
            CommandOptionType::Mentionable => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::Mentionable,
                description_localizations: Default::default(),
                name_localizations: Default::default(),
                autocomplete: None,
                channel_types: None,
                choices: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                options: None,
            },
            CommandOptionType::Number => Self {
                name: v.name,
                description: v.description,
                required: Some(v.required),
                kind: twilight_model::application::command::CommandOptionType::Number,
                min_value: v
                    .extra_options
                    .min_value
                    .map(|v| CommandOptionValue::Number(v)),
                max_value: v
                    .extra_options
                    .max_value
                    .map(|v| CommandOptionValue::Number(v)),
                autocomplete: None,
                channel_types: None,
                choices: None,
                description_localizations: None,
                max_length: None,
                min_length: None,
                name_localizations: None,
                options: None,
            },
        }
    }
}
