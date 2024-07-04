use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::id::Id;

use super::channel::ChannelType;
use super::message::ReactionType;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/ComponentType.ts")]
pub enum ComponentType {
    ActionRow,
    Button,
    SelectMenu,
    TextInput,
    UserSelectMenu,
    RoleSelectMenu,
    MentionableSelectMenu,
    ChannelSelectMenu,
}

use twilight_model::channel::message::component::ComponentType as TwilightComponentType;
impl From<TwilightComponentType> for ComponentType {
    fn from(v: TwilightComponentType) -> Self {
        match v {
            TwilightComponentType::ActionRow => Self::ActionRow,
            TwilightComponentType::Button => Self::Button,
            TwilightComponentType::TextSelectMenu => Self::SelectMenu,
            TwilightComponentType::UserSelectMenu => Self::UserSelectMenu,
            TwilightComponentType::RoleSelectMenu => Self::RoleSelectMenu,
            TwilightComponentType::MentionableSelectMenu => Self::MentionableSelectMenu,
            TwilightComponentType::ChannelSelectMenu => Self::ChannelSelectMenu,
            TwilightComponentType::TextInput => Self::TextInput,
            _ => todo!(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IComponent",
    export_to = "bindings/discord/IComponent.ts"
)]
// #[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
    TextInput(TextInput),
    UserSelectMenu(SelectMenu),
    RoleSelectMenu(SelectMenu),
    MentionableSelectMenu(SelectMenu),
    ChannelSelectMenu(SelectMenu),

    Unknown(UnknownComponent),
}

use twilight_model::channel::message::component::Component as TwilightComponent;
impl From<TwilightComponent> for Component {
    fn from(v: TwilightComponent) -> Self {
        match v {
            TwilightComponent::ActionRow(inner) => Self::ActionRow(inner.into()),
            TwilightComponent::Button(inner) => Self::Button(inner.into()),
            TwilightComponent::SelectMenu(inner) => match inner.kind {
                TwilightSelectMenuType::Text => Self::SelectMenu(inner.into()),
                TwilightSelectMenuType::User => Self::UserSelectMenu(inner.into()),
                TwilightSelectMenuType::Role => Self::RoleSelectMenu(inner.into()),
                TwilightSelectMenuType::Mentionable => Self::MentionableSelectMenu(inner.into()),
                TwilightSelectMenuType::Channel => Self::ChannelSelectMenu(inner.into()),
                _ => todo!(),
            },
            TwilightComponent::TextInput(inner) => Self::TextInput(inner.into()),
            TwilightComponent::Unknown(t) => Self::Unknown(UnknownComponent { component_kind: t }),
        }
    }
}
impl TryFrom<Component> for TwilightComponent {
    type Error = anyhow::Error;

    fn try_from(v: Component) -> Result<Self, Self::Error> {
        Ok(match v {
            Component::ActionRow(inner) => Self::ActionRow(inner.try_into()?),
            Component::Button(inner) => Self::Button(inner.into()),
            Component::SelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::TextInput(inner) => Self::TextInput(inner.into()),
            Component::Unknown(c) => Self::Unknown(c.component_kind),
            Component::UserSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::RoleSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::MentionableSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::ChannelSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IUnknownComponent",
    export_to = "bindings/discord/IUnknownComponent.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct UnknownComponent {
    pub component_kind: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IActionRow",
    export_to = "bindings/discord/IActionRow.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct ActionRow {
    pub components: Vec<Component>,
}

use twilight_model::channel::message::component::ActionRow as TwilightActionRow;
impl From<TwilightActionRow> for ActionRow {
    fn from(v: TwilightActionRow) -> Self {
        Self {
            components: v.components.into_iter().map(Into::into).collect(),
        }
    }
}
impl TryFrom<ActionRow> for TwilightActionRow {
    type Error = anyhow::Error;

    fn try_from(v: ActionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            components: v
                .components
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, rename = "IButton", export_to = "bindings/discord/IButton.ts")]
#[serde(rename_all = "camelCase")]
pub struct Button {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    pub style: ButtonStyle,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ReactionType>,
}

use twilight_model::channel::message::component::Button as TwilightButton;
impl From<TwilightButton> for Button {
    fn from(v: TwilightButton) -> Self {
        Self {
            custom_id: v.custom_id,
            disabled: Some(v.disabled),
            style: v.style.into(),
            url: v.url,
            label: v.label,
            emoji: v.emoji.map(Into::into),
        }
    }
}

impl From<Button> for TwilightButton {
    fn from(v: Button) -> Self {
        Self {
            custom_id: v.custom_id,
            disabled: v.disabled.unwrap_or_default(),
            style: v.style.into(),
            url: v.url,
            label: v.label,
            emoji: v.emoji.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ISelectMenu",
    export_to = "bindings/discord/ISelectMenu.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct SelectMenu {
    pub custom_id: String,
    pub disabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_values: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_values: Option<u8>,

    pub options: Vec<SelectMenuOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    pub select_type: SelectMenuType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_types: Option<Vec<ChannelType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_values: Option<Vec<SelectDefaultValue>>,
}

use twilight_model::channel::message::component::SelectDefaultValue as TwilightSelectDefaultValue;
use twilight_model::channel::message::component::SelectMenu as TwilightSelectMenu;
use twilight_model::channel::message::component::SelectMenuType as TwilightSelectMenuType;

impl From<TwilightSelectMenu> for SelectMenu {
    fn from(v: TwilightSelectMenu) -> Self {
        Self {
            custom_id: v.custom_id,
            disabled: v.disabled,
            min_values: v.min_values,
            max_values: v.max_values,
            options: v
                .options
                .map(|v| v.into_iter().map(Into::into).collect())
                .unwrap_or_default(),
            placeholder: v.placeholder,
            channel_types: v
                .channel_types
                .map(|v| v.into_iter().map(Into::into).collect()),
            default_values: v
                .default_values
                .map(|v| v.into_iter().map(Into::into).collect()),
            select_type: v.kind.into(),
        }
    }
}
impl TryFrom<SelectMenu> for TwilightSelectMenu {
    type Error = anyhow::Error;

    fn try_from(v: SelectMenu) -> Result<Self, Self::Error> {
        Ok(Self {
            custom_id: v.custom_id,
            disabled: v.disabled,
            min_values: v.min_values,
            max_values: v.max_values,
            options: if !v.options.is_empty() {
                Some(v.options.into_iter().map(Into::into).collect())
            } else {
                None
            },
            placeholder: v.placeholder,
            channel_types: v
                .channel_types
                .map(|v| v.into_iter().map(Into::into).collect()),
            default_values: v
                .default_values
                .map(|v| {
                    v.into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_, _>>()
                })
                .transpose()?,
            kind: v.select_type.into(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "SelectDefaultValue",
    export_to = "bindings/discord/SelectDefaultValue.ts"
)]
#[serde(tag = "kind", content = "value")]
pub enum SelectDefaultValue {
    User(String),
    Role(String),
    Channel(String),
}

impl From<TwilightSelectDefaultValue> for SelectDefaultValue {
    fn from(value: TwilightSelectDefaultValue) -> Self {
        match value {
            TwilightSelectDefaultValue::User(id) => Self::User(id.to_string()),
            TwilightSelectDefaultValue::Role(id) => Self::Role(id.to_string()),
            TwilightSelectDefaultValue::Channel(id) => Self::Channel(id.to_string()),
        }
    }
}

impl TryFrom<SelectDefaultValue> for TwilightSelectDefaultValue {
    type Error = anyhow::Error;
    fn try_from(value: SelectDefaultValue) -> Result<Self, Self::Error> {
        match value {
            SelectDefaultValue::User(id) => Ok(Self::User(
                Id::new_checked(id.parse()?).ok_or_else(|| anyhow::anyhow!("Bad snowflake"))?,
            )),
            SelectDefaultValue::Role(id) => Ok(Self::Role(
                Id::new_checked(id.parse()?).ok_or_else(|| anyhow::anyhow!("Bad snowflake"))?,
            )),
            SelectDefaultValue::Channel(id) => Ok(Self::Channel(
                Id::new_checked(id.parse()?).ok_or_else(|| anyhow::anyhow!("Bad snowflake"))?,
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "SelectMenuType",
    export_to = "bindings/discord/SelectMenuType.ts"
)]
pub enum SelectMenuType {
    Text,
    User,
    Role,
    Mentionable,
    Channel,
}

impl From<TwilightSelectMenuType> for SelectMenuType {
    fn from(value: TwilightSelectMenuType) -> Self {
        match value {
            TwilightSelectMenuType::Text => SelectMenuType::Text,
            TwilightSelectMenuType::User => SelectMenuType::User,
            TwilightSelectMenuType::Role => SelectMenuType::Role,
            TwilightSelectMenuType::Mentionable => SelectMenuType::Mentionable,
            TwilightSelectMenuType::Channel => SelectMenuType::Channel,
            _ => todo!(),
        }
    }
}

impl From<SelectMenuType> for TwilightSelectMenuType {
    fn from(value: SelectMenuType) -> Self {
        match value {
            SelectMenuType::Text => Self::Text,
            SelectMenuType::User => Self::User,
            SelectMenuType::Role => Self::Role,
            SelectMenuType::Mentionable => Self::Mentionable,
            SelectMenuType::Channel => Self::Channel,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ISelectMenuOption",
    export_to = "bindings/discord/ISelectMenuOption.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct SelectMenuOption {
    pub default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ReactionType>,
    pub label: String,
    pub value: String,
}

use twilight_model::channel::message::component::SelectMenuOption as TwilightSelectMenuOption;
impl From<TwilightSelectMenuOption> for SelectMenuOption {
    fn from(v: TwilightSelectMenuOption) -> Self {
        Self {
            default: v.default,
            description: v.description,
            emoji: v.emoji.map(Into::into),
            label: v.label,
            value: v.value,
        }
    }
}
impl From<SelectMenuOption> for TwilightSelectMenuOption {
    fn from(v: SelectMenuOption) -> Self {
        Self {
            default: v.default,
            description: v.description,
            emoji: v.emoji.map(Into::into),
            label: v.label,
            value: v.value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/ButtonStyle.ts")]
// #[serde(rename_all = "camelCase")]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Link,
    Premium,
}

use twilight_model::channel::message::component::ButtonStyle as TwilightButtonStyle;
impl From<TwilightButtonStyle> for ButtonStyle {
    fn from(v: TwilightButtonStyle) -> Self {
        match v {
            TwilightButtonStyle::Primary => Self::Primary,
            TwilightButtonStyle::Secondary => Self::Secondary,
            TwilightButtonStyle::Success => Self::Success,
            TwilightButtonStyle::Danger => Self::Danger,
            TwilightButtonStyle::Link => Self::Link,
            TwilightButtonStyle::Unknown(6) => Self::Premium,
            _ => todo!(),
        }
    }
}
impl From<ButtonStyle> for TwilightButtonStyle {
    fn from(v: ButtonStyle) -> Self {
        match v {
            ButtonStyle::Primary => Self::Primary,
            ButtonStyle::Secondary => Self::Secondary,
            ButtonStyle::Success => Self::Success,
            ButtonStyle::Danger => Self::Danger,
            ButtonStyle::Link => Self::Link,
            ButtonStyle::Premium => Self::Unknown(6),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ITextInput",
    export_to = "bindings/discord/ITextInput.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct TextInput {
    pub custom_id: String,
    pub label: String,
    pub max_length: Option<u16>,
    pub min_length: Option<u16>,
    pub placeholder: Option<String>,
    pub required: Option<bool>,
    pub style: TextInputStyle,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/TextInputStyle.ts")]
pub enum TextInputStyle {
    Short,
    Paragraph,
}

use twilight_model::channel::message::component::TextInput as TwilightTextInput;
impl From<TwilightTextInput> for TextInput {
    fn from(v: TwilightTextInput) -> Self {
        Self {
            custom_id: v.custom_id,
            label: v.label,
            max_length: v.max_length,
            min_length: v.min_length,
            placeholder: v.placeholder,
            required: v.required,
            style: v.style.into(),
            value: v.value,
        }
    }
}

impl From<TextInput> for TwilightTextInput {
    fn from(v: TextInput) -> Self {
        Self {
            custom_id: v.custom_id,
            label: v.label,
            max_length: v.max_length,
            min_length: v.min_length,
            placeholder: v.placeholder,
            required: v.required,
            style: v.style.into(),
            value: v.value,
        }
    }
}

use twilight_model::channel::message::component::TextInputStyle as TwilightTextInputStyle;
impl From<TwilightTextInputStyle> for TextInputStyle {
    fn from(v: TwilightTextInputStyle) -> Self {
        match v {
            TwilightTextInputStyle::Short => Self::Short,
            TwilightTextInputStyle::Paragraph => Self::Paragraph,
            _ => todo!(),
        }
    }
}

impl From<TextInputStyle> for TwilightTextInputStyle {
    fn from(v: TextInputStyle) -> Self {
        match v {
            TextInputStyle::Short => Self::Short,
            TextInputStyle::Paragraph => Self::Paragraph,
        }
    }
}
