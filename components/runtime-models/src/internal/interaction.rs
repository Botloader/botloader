use super::{member::Member, messages::Message, script::CommandOptionType};
use crate::{
    discord::{
        component::{ComponentType, UnsupportedComponent},
        message::Attachment,
    },
    util::NotBigI64,
};
use serde::Serialize;
use ts_rs::TS;
use twilight_model::id::Id;

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
                component_type: data.component_type.try_into()?,
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
                    values: convert_modal_components_lossy(data.components)?,
                    resolved: data.resolved.map(|rv| rv.try_into()).transpose()?,
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
                .map(|(k, v)| Ok((k.to_string(), v.try_into()?)))
                .collect::<Result<_, anyhow::Error>>()?,
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
    pub resolved: Option<InteractionDataMap>,

    pub custom_id: String,
    pub values: Vec<ModalInteractionComponent>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "bindings/internal/ModalInteractionComponent.ts")]
pub enum ModalInteractionComponent {
    Label(ModalInteractionLabel),
    ActionRow(ModalInteractionActionRow),
    SelectMenu(ModalInteractionStringSelect),
    UserSelectMenu(ModalInteractionUserSelect),
    RoleSelectMenu(ModalInteractionRoleSelect),
    MentionableSelectMenu(ModalInteractionMentionableSelect),
    ChannelSelectMenu(ModalInteractionChannelSelect),
    TextInput(ModalInteractionTextInput),
    TextDisplay(ModalInteractionTextDisplay),
    FileUpload(ModalInteractionFileUpload),
    Checkbox(ModalInteractionCheckbox),
    CheckboxGroup(ModalInteractionCheckboxGroup),
}

use twilight_model::application::interaction::modal::ModalInteractionComponent as TwilightModalInteractionComponent;

/// Converts a list of incoming modal components, dropping components that
/// can't be represented in the API (component types Discord added after this
/// was built) instead of failing the whole conversion.
fn convert_modal_components_lossy(
    components: Vec<TwilightModalInteractionComponent>,
) -> anyhow::Result<Vec<ModalInteractionComponent>> {
    components
        .into_iter()
        .filter_map(|c| match ModalInteractionComponent::try_from(c) {
            Err(err) if err.downcast_ref::<UnsupportedComponent>().is_some() => {
                tracing::info!("dropping {err}");
                None
            }
            other => Some(other),
        })
        .collect()
}

impl TryFrom<TwilightModalInteractionComponent> for ModalInteractionComponent {
    type Error = anyhow::Error;

    fn try_from(v: TwilightModalInteractionComponent) -> Result<Self, Self::Error> {
        Ok(match v {
            TwilightModalInteractionComponent::Label(inner) => Self::Label(inner.try_into()?),
            TwilightModalInteractionComponent::ActionRow(inner) => Self::ActionRow(inner.try_into()?),
            TwilightModalInteractionComponent::StringSelect(inner) => Self::SelectMenu(inner.try_into()?),
            TwilightModalInteractionComponent::UserSelect(inner) => Self::UserSelectMenu(inner.try_into()?),
            TwilightModalInteractionComponent::RoleSelect(inner) => Self::RoleSelectMenu(inner.try_into()?),
            TwilightModalInteractionComponent::MentionableSelect(inner) => Self::MentionableSelectMenu(inner.try_into()?),
            TwilightModalInteractionComponent::ChannelSelect(inner) => Self::ChannelSelectMenu(inner.try_into()?),
            TwilightModalInteractionComponent::TextInput(inner) => Self::TextInput(inner.try_into()?),
            TwilightModalInteractionComponent::TextDisplay(inner) => Self::TextDisplay(inner.try_into()?),
            TwilightModalInteractionComponent::FileUpload(inner) => Self::FileUpload(inner.try_into()?),
            TwilightModalInteractionComponent::Checkbox(inner) => Self::Checkbox(inner.try_into()?),
            TwilightModalInteractionComponent::CheckboxGroup(inner) => Self::CheckboxGroup(inner.try_into()?),
            TwilightModalInteractionComponent::Unknown(inner) => {
                return Err(UnsupportedComponent(format!("modal component type {inner}")).into())
            }
        })
    }
}

impl TryFrom<ModalInteractionComponent> for TwilightModalInteractionComponent {
    type Error = anyhow::Error;

    fn try_from(v: ModalInteractionComponent) -> Result<Self, Self::Error> {
        Ok(match v {
            ModalInteractionComponent::Label(inner) => Self::Label(inner.try_into()?),
            ModalInteractionComponent::ActionRow(inner) => Self::ActionRow(inner.try_into()?),
            ModalInteractionComponent::SelectMenu(inner) => Self::StringSelect(inner.try_into()?),
            ModalInteractionComponent::UserSelectMenu(inner) => Self::UserSelect(inner.try_into()?),
            ModalInteractionComponent::RoleSelectMenu(inner) => Self::RoleSelect(inner.try_into()?),
            ModalInteractionComponent::MentionableSelectMenu(inner) => Self::MentionableSelect(inner.try_into()?),
            ModalInteractionComponent::ChannelSelectMenu(inner) => Self::ChannelSelect(inner.try_into()?),
            ModalInteractionComponent::TextInput(inner) => Self::TextInput(inner.try_into()?),
            ModalInteractionComponent::TextDisplay(inner) => Self::TextDisplay(inner.try_into()?),
            ModalInteractionComponent::FileUpload(inner) => Self::FileUpload(inner.try_into()?),
            ModalInteractionComponent::Checkbox(inner) => Self::Checkbox(inner.try_into()?),
            ModalInteractionComponent::CheckboxGroup(inner) => Self::CheckboxGroup(inner.try_into()?),
        })
    }
}


#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionLabel.ts",
    rename = "IModalInteractionLabel"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionLabel {
    pub id: i32,
    pub component: Box<ModalInteractionComponent>,
}

use twilight_model::application::interaction::modal::ModalInteractionLabel as TwilightModalInteractionLabel;
impl TryFrom<TwilightModalInteractionLabel> for ModalInteractionLabel {
    type Error = anyhow::Error;

    fn try_from(v: TwilightModalInteractionLabel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            component: Box::new((*v.component).try_into()?),
        })
    }
}

impl TryFrom<ModalInteractionLabel> for TwilightModalInteractionLabel {
    type Error = anyhow::Error;

    fn try_from(v: ModalInteractionLabel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            component: Box::new((*v.component).try_into()?),
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionActionRow.ts",
    rename = "IModalInteractionActionRow"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionActionRow {
    pub id: i32,
    pub components: Vec<ModalInteractionComponent>,
}

use twilight_model::application::interaction::modal::ModalInteractionActionRow as TwilightModalInteractionActionRow;
impl TryFrom<TwilightModalInteractionActionRow> for ModalInteractionActionRow {
    type Error = anyhow::Error;

    fn try_from(v: TwilightModalInteractionActionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            components: convert_modal_components_lossy(v.components)?,
        })
    }
}

impl TryFrom<ModalInteractionActionRow> for TwilightModalInteractionActionRow {
    type Error = anyhow::Error;

    fn try_from(v: ModalInteractionActionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            components: v
                .components
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionSelectMenu.ts",
    rename = "IModalInteractionSelectMenu"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionSelectMenu<ValueType> {
    pub id: i32,
    pub custom_id: String,
    pub values: Vec<ValueType>,
}

type ModalInteractionStringSelect = ModalInteractionSelectMenu<String>;

use twilight_model::application::interaction::modal::ModalInteractionStringSelect as TwilightModalInteractionStringSelect;
impl TryFrom<TwilightModalInteractionStringSelect> for ModalInteractionStringSelect {
    type Error = anyhow::Error;

    fn try_from(v: TwilightModalInteractionStringSelect) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<ModalInteractionStringSelect> for TwilightModalInteractionStringSelect {
    type Error = anyhow::Error;

    fn try_from(v: ModalInteractionStringSelect) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

type ModalInteractionUserSelect = ModalInteractionSelectMenu<String>;
use twilight_model::application::interaction::modal::ModalInteractionUserSelect as TwilightModalInteractionUserSelect;
impl From<TwilightModalInteractionUserSelect> for ModalInteractionUserSelect {
    fn from(v: TwilightModalInteractionUserSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }
}

impl From<ModalInteractionUserSelect> for TwilightModalInteractionUserSelect {
    fn from(v: ModalInteractionUserSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
        }
    }
}

type ModalInteractionRoleSelect = ModalInteractionSelectMenu<String>;
use twilight_model::application::interaction::modal::ModalInteractionRoleSelect as TwilightModalInteractionRoleSelect;
impl From<TwilightModalInteractionRoleSelect> for ModalInteractionRoleSelect {
    fn from(v: TwilightModalInteractionRoleSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }
}

impl From<ModalInteractionRoleSelect> for TwilightModalInteractionRoleSelect {
    fn from(v: ModalInteractionRoleSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
        }
    }
}

type ModalInteractionMentionableSelect = ModalInteractionSelectMenu<String>;
use twilight_model::application::interaction::modal::ModalInteractionMentionableSelect as TwilightModalInteractionMentionableSelect;
impl From<TwilightModalInteractionMentionableSelect> for ModalInteractionMentionableSelect {
    fn from(v: TwilightModalInteractionMentionableSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }
}

impl From<ModalInteractionMentionableSelect> for TwilightModalInteractionMentionableSelect{
    fn from(v: ModalInteractionMentionableSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
        }
    }
}

type ModalInteractionChannelSelect = ModalInteractionSelectMenu<String>;
use twilight_model::application::interaction::modal::ModalInteractionChannelSelect as TwilightModalInteractionChannelSelect;
impl From<TwilightModalInteractionChannelSelect> for ModalInteractionChannelSelect {
    fn from(v: TwilightModalInteractionChannelSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }
}

impl From<ModalInteractionChannelSelect> for TwilightModalInteractionChannelSelect {
    fn from(v: ModalInteractionChannelSelect) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionTextInput.ts",
    rename = "IModalInteractionTextInput"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionTextInput {
    pub custom_id: String,
    pub id: i32,
    pub value: String,
}

use twilight_model::application::interaction::modal::ModalInteractionTextInput as TwilightModalInteractionTextInput;
impl From<TwilightModalInteractionTextInput> for ModalInteractionTextInput {
    fn from(v: TwilightModalInteractionTextInput) -> Self {
        Self {
            custom_id: v.custom_id,
            id: v.id,
            value: v.value,
        }
    }
}

impl From<ModalInteractionTextInput> for TwilightModalInteractionTextInput {
    fn from(v: ModalInteractionTextInput) -> Self {
        Self {
            custom_id: v.custom_id,
            id: v.id,
            value: v.value,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionTextDisplay.ts",
    rename = "IModalInteractionTextDisplay"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionTextDisplay {
    pub id: i32,
}

use twilight_model::application::interaction::modal::ModalInteractionTextDisplay as TwilightModalInteractionTextDisplay;
impl From<TwilightModalInteractionTextDisplay> for ModalInteractionTextDisplay {
    fn from(v: TwilightModalInteractionTextDisplay) -> Self {
        Self {
            id: v.id,
        }
    }
}

impl From<ModalInteractionTextDisplay> for TwilightModalInteractionTextDisplay {
    fn from(v: ModalInteractionTextDisplay) -> Self {
        Self {
            id: v.id,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionFileUpload.ts",
    rename = "IModalInteractionFileUpload"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionFileUpload {
    pub id: i32,
    pub custom_id: String,
    pub values: Vec<String>,
}

use twilight_model::application::interaction::modal::ModalInteractionFileUpload as TwilightModalInteractionFileUpload;
impl From<TwilightModalInteractionFileUpload> for ModalInteractionFileUpload {
    fn from(v: TwilightModalInteractionFileUpload) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }
}

impl From<ModalInteractionFileUpload> for TwilightModalInteractionFileUpload {
    fn from(v: ModalInteractionFileUpload) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .iter()
                .filter_map(|s| Id::new_checked(s.parse().ok()?))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionCheckbox.ts",
    rename = "ModalInteractionCheckbox"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionCheckbox {
    pub id: i32,
    pub custom_id: String,
    pub value: bool,
}

use twilight_model::application::interaction::modal::ModalInteractionCheckbox as TwilightModalInteractionCheckbox;
impl From<TwilightModalInteractionCheckbox> for ModalInteractionCheckbox {
    fn from(v: TwilightModalInteractionCheckbox) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            value: v.value,
        }
    }
}

impl From<ModalInteractionCheckbox> for TwilightModalInteractionCheckbox {
    fn from(v: ModalInteractionCheckbox) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            value: v.value,
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IModalInteractionCheckboxGroup.ts",
    rename = "ModalInteractionCheckboxGroup"
)]
#[serde(rename_all = "camelCase")]
pub struct ModalInteractionCheckboxGroup {
    pub id: i32,
    pub custom_id: String,
    pub values: Vec<String>,
}

use twilight_model::application::interaction::modal::ModalInteractionCheckboxGroup as TwilightModalInteractionCheckboxGroup;
impl From<TwilightModalInteractionCheckboxGroup> for ModalInteractionCheckboxGroup {
    fn from(v: TwilightModalInteractionCheckboxGroup) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<ModalInteractionCheckboxGroup> for TwilightModalInteractionCheckboxGroup {
    fn from(v: ModalInteractionCheckboxGroup) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            values: v
                .values
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/InteractionType.ts")]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutocomplete = 4,
    ModalSubmit = 5
}

use twilight_model::application::interaction::InteractionType as TwilightInteractionType;
impl From<TwilightInteractionType> for InteractionType {
    fn from(v: TwilightInteractionType) -> Self {
        match v {
            TwilightInteractionType::Ping => Self::Ping,
            TwilightInteractionType::ApplicationCommand => Self::ApplicationCommand,
            TwilightInteractionType::MessageComponent => Self::MessageComponent,
            TwilightInteractionType::ApplicationCommandAutocomplete => Self::ApplicationCommandAutocomplete,
            TwilightInteractionType::ModalSubmit => Self::ModalSubmit,
            _ => todo!(),
        }
    }
}


#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "bindings/internal/IInteractionMetadata.ts",
    rename = "IInteractionMetadata"
)]
#[serde(rename_all = "camelCase")]
pub struct InteractionMetadata {
    pub id: String,
    pub interacted_message_id: Option<String>,
    pub kind: InteractionType,
    pub original_response_message_id: Option<String>,
    pub target_message_id: Option<String>,
    pub target_user: Option<User>,
    pub triggering_interaction_metadata: Option<Box<InteractionMetadata>>,
    pub user: User,
}

impl From<twilight_model::application::interaction::InteractionMetadata> for InteractionMetadata {
    fn from(v: twilight_model::application::interaction::InteractionMetadata) -> Self {
        Self {
            id: v.id.to_string(),
            interacted_message_id: v
                .interacted_message_id
                .as_ref()
                .map(ToString::to_string),
            kind: v.kind.into(),
            original_response_message_id: v
                .original_response_message_id
                .as_ref()
                .map(ToString::to_string),
            target_message_id: v
                .target_message_id.as_ref()
                .map(ToString::to_string),
            target_user: v.target_user.map(From::from),
            triggering_interaction_metadata: v
                .triggering_interaction_metadata
                .map(|e| Box::new((*e).into())),
            user: v.user.into()
        }
    }
}
