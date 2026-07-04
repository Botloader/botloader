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
    Section,
    TextDisplay,
    Thumbnail,
    MediaGallery,
    File,
    Separator,
    Container,
    Label,
    FileUpload,
    Checkbox,
    CheckboxGroup,
}

use twilight_model::channel::message::component::ComponentType as TwilightComponentType;
impl TryFrom<TwilightComponentType> for ComponentType {
    type Error = anyhow::Error;

    fn try_from(v: TwilightComponentType) -> Result<Self, Self::Error> {
        Ok(match v {
            TwilightComponentType::ActionRow => Self::ActionRow,
            TwilightComponentType::Button => Self::Button,
            TwilightComponentType::TextSelectMenu => Self::SelectMenu,
            TwilightComponentType::UserSelectMenu => Self::UserSelectMenu,
            TwilightComponentType::RoleSelectMenu => Self::RoleSelectMenu,
            TwilightComponentType::MentionableSelectMenu => Self::MentionableSelectMenu,
            TwilightComponentType::ChannelSelectMenu => Self::ChannelSelectMenu,
            TwilightComponentType::TextInput => Self::TextInput,
            TwilightComponentType::Section => Self::Section,
            TwilightComponentType::TextDisplay => Self::TextDisplay,
            TwilightComponentType::Thumbnail => Self::Thumbnail,
            TwilightComponentType::MediaGallery => Self::MediaGallery,
            TwilightComponentType::File => Self::File,
            TwilightComponentType::Separator => Self::Separator,
            TwilightComponentType::Container => Self::Container,
            TwilightComponentType::Label => Self::Label,
            TwilightComponentType::FileUpload => Self::FileUpload,
            TwilightComponentType::Checkbox => Self::Checkbox,
            TwilightComponentType::CheckboxGroup => Self::CheckboxGroup,
            TwilightComponentType::Unknown(n) => {
                return Err(UnsupportedComponent(format!("component type {n}")).into())
            }
            other => {
                return Err(UnsupportedComponent(format!("component type {other:?}")).into())
            }
        })
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
    Section(Section),
    TextDisplay(TextDisplay),
    Thumbnail(Thumbnail),
    MediaGallery(MediaGallery),
    File(FileDisplay),
    Separator(Separator),
    Container(Container),
    Label(Label),
    FileUpload(FileUpload),
    Checkbox(Checkbox),
    CheckboxGroup(CheckboxGroup),
}

/// Error returned when converting an incoming component (or one of its
/// required children) that this API has no representation for.
///
/// List conversion sites use [`convert_components_lossy`] to drop these
/// instead of failing the whole conversion.
#[derive(Debug, Clone)]
pub struct UnsupportedComponent(pub String);

impl std::fmt::Display for UnsupportedComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported component: {}", self.0)
    }
}

impl std::error::Error for UnsupportedComponent {}

/// Converts a list of incoming components, dropping components that can't be
/// represented in the API (component types Discord added after this was
/// built) instead of failing the whole conversion.
pub fn convert_components_lossy(
    components: Vec<TwilightComponent>,
) -> anyhow::Result<Vec<Component>> {
    components
        .into_iter()
        .filter_map(|c| match Component::try_from(c) {
            Err(err) if err.downcast_ref::<UnsupportedComponent>().is_some() => {
                tracing::info!("dropping {err}");
                None
            }
            other => Some(other),
        })
        .collect()
}

use twilight_model::channel::message::component::Component as TwilightComponent;
impl TryFrom<TwilightComponent> for Component {
    type Error = anyhow::Error;

    fn try_from(v: TwilightComponent) -> Result<Self, Self::Error> {
        match v {
            TwilightComponent::ActionRow(inner) => Ok(Self::ActionRow(inner.try_into()?)),
            TwilightComponent::Button(inner) => Ok(Self::Button(inner.try_into()?)),
            TwilightComponent::SelectMenu(inner) => match inner.kind {
                TwilightSelectMenuType::Text => Ok(Self::SelectMenu(inner.try_into()?)),
                TwilightSelectMenuType::User => Ok(Self::UserSelectMenu(inner.try_into()?)),
                TwilightSelectMenuType::Role => Ok(Self::RoleSelectMenu(inner.try_into()?)),
                TwilightSelectMenuType::Mentionable => {
                    Ok(Self::MentionableSelectMenu(inner.try_into()?))
                }
                TwilightSelectMenuType::Channel => Ok(Self::ChannelSelectMenu(inner.try_into()?)),
                other => Err(UnsupportedComponent(format!("select menu kind {other:?}")).into()),
            },
            TwilightComponent::TextInput(inner) => Ok(Self::TextInput(inner.try_into()?)),
            TwilightComponent::TextDisplay(inner) => Ok(Self::TextDisplay(inner.try_into()?)),
            TwilightComponent::MediaGallery(inner) => Ok(Self::MediaGallery(inner.try_into()?)),
            TwilightComponent::Separator(inner) => Ok(Self::Separator(inner.try_into()?)),
            TwilightComponent::File(inner) => Ok(Self::File(inner.try_into()?)),
            TwilightComponent::Section(inner) => Ok(Self::Section(inner.try_into()?)),
            TwilightComponent::Container(inner) => Ok(Self::Container(inner.try_into()?)),
            TwilightComponent::Thumbnail(inner) => Ok(Self::Thumbnail(inner.try_into()?)),
            TwilightComponent::Label(inner) => Ok(Self::Label(inner.try_into()?)),
            TwilightComponent::FileUpload(inner) => Ok(Self::FileUpload(inner.try_into()?)),
            TwilightComponent::Checkbox(inner) => Ok(Self::Checkbox(inner.try_into()?)),
            TwilightComponent::CheckboxGroup(inner) => Ok(Self::CheckboxGroup(inner.try_into()?)),
            TwilightComponent::Unknown(t) => {
                Err(UnsupportedComponent(format!("component type {t}")).into())
            }
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
            Component::UserSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::RoleSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::MentionableSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::ChannelSelectMenu(inner) => Self::SelectMenu(inner.try_into()?),
            Component::Section(inner) => Self::Section(inner.try_into()?),
            Component::TextDisplay(inner) => Self::TextDisplay(inner.try_into()?),
            Component::Thumbnail(inner) => Self::Thumbnail(inner.try_into()?),
            Component::MediaGallery(inner) => Self::MediaGallery(inner.try_into()?),
            Component::File(inner) => Self::File(inner.try_into()?),
            Component::Separator(inner) => Self::Separator(inner.try_into()?),
            Component::Container(inner) => Self::Container(inner.try_into()?),
            Component::Label(inner) => Self::Label(inner.try_into()?),
            Component::FileUpload(inner) => Self::FileUpload(inner.try_into()?),
            Component::Checkbox(inner) => Self::Checkbox(inner.try_into()?),
            Component::CheckboxGroup(inner) => Self::CheckboxGroup(inner.try_into()?),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IActionRow",
    export_to = "bindings/discord/IActionRow.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct ActionRow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub components: Vec<Component>,
}

use twilight_model::channel::message::component::ActionRow as TwilightActionRow;
impl TryFrom<TwilightActionRow> for ActionRow {
    type Error = anyhow::Error;

    fn try_from(v: TwilightActionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            components: convert_components_lossy(v.components)?,
        })
    }
}
impl TryFrom<ActionRow> for TwilightActionRow {
    type Error = anyhow::Error;

    fn try_from(v: ActionRow) -> Result<Self, Self::Error> {
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

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, rename = "IButton", export_to = "bindings/discord/IButton.ts")]
#[serde(rename_all = "camelCase")]
pub struct Button {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub custom_id: Option<String>,
    pub style: ButtonStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub emoji: Option<ReactionType>,
}

use twilight_model::channel::message::component::Button as TwilightButton;
impl TryFrom<TwilightButton> for Button {
    type Error = anyhow::Error;

    fn try_from(v: TwilightButton) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            custom_id: v.custom_id,
            disabled: Some(v.disabled),
            style: v.style.try_into()?,
            url: v.url,
            label: v.label,
            emoji: v.emoji.map(Into::into),
        })
    }
}

impl From<Button> for TwilightButton {
    fn from(v: Button) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            disabled: v.disabled.unwrap_or_default(),
            style: v.style.into(),
            url: v.url,
            label: v.label,
            emoji: v.emoji.map(Into::into),
            sku_id: None,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,

    pub custom_id: String,
    pub disabled: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_values: Option<u8>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_values: Option<u8>,

    pub options: Vec<SelectMenuOption>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub placeholder: Option<String>,

    pub select_type: SelectMenuType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub channel_types: Option<Vec<ChannelType>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default_values: Option<Vec<SelectDefaultValue>>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub required: Option<bool>,
}

use twilight_model::channel::message::component::SelectDefaultValue as TwilightSelectDefaultValue;
use twilight_model::channel::message::component::SelectMenu as TwilightSelectMenu;
use twilight_model::channel::message::component::SelectMenuType as TwilightSelectMenuType;

impl TryFrom<TwilightSelectMenu> for SelectMenu {
    type Error = anyhow::Error;

    fn try_from(v: TwilightSelectMenu) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
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
                .map(|v| {
                    v.into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
            default_values: v
                .default_values
                .map(|v| v.into_iter().map(Into::into).collect()),
            select_type: v.kind.try_into()?,
            required: v.required,
        })
    }
}
impl TryFrom<SelectMenu> for TwilightSelectMenu {
    type Error = anyhow::Error;

    fn try_from(v: SelectMenu) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
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
            required: v.required,
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

impl TryFrom<TwilightSelectMenuType> for SelectMenuType {
    type Error = anyhow::Error;

    fn try_from(value: TwilightSelectMenuType) -> Result<Self, Self::Error> {
        Ok(match value {
            TwilightSelectMenuType::Text => SelectMenuType::Text,
            TwilightSelectMenuType::User => SelectMenuType::User,
            TwilightSelectMenuType::Role => SelectMenuType::Role,
            TwilightSelectMenuType::Mentionable => SelectMenuType::Mentionable,
            TwilightSelectMenuType::Channel => SelectMenuType::Channel,
            other => {
                return Err(UnsupportedComponent(format!("select menu kind {other:?}")).into())
            }
        })
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
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
impl TryFrom<TwilightButtonStyle> for ButtonStyle {
    type Error = anyhow::Error;

    fn try_from(v: TwilightButtonStyle) -> Result<Self, Self::Error> {
        Ok(match v {
            TwilightButtonStyle::Primary => Self::Primary,
            TwilightButtonStyle::Secondary => Self::Secondary,
            TwilightButtonStyle::Success => Self::Success,
            TwilightButtonStyle::Danger => Self::Danger,
            TwilightButtonStyle::Link => Self::Link,
            TwilightButtonStyle::Premium => Self::Premium,
            other => return Err(UnsupportedComponent(format!("button style {other:?}")).into()),
        })
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
            ButtonStyle::Premium => Self::Premium,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub custom_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_length: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_length: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub required: Option<bool>,
    pub style: TextInputStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/TextInputStyle.ts")]
pub enum TextInputStyle {
    Short,
    Paragraph,
}

use twilight_model::channel::message::component::TextInput as TwilightTextInput;
impl TryFrom<TwilightTextInput> for TextInput {
    type Error = anyhow::Error;

    fn try_from(v: TwilightTextInput) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            custom_id: v.custom_id,
            label: v.label,
            max_length: v.max_length,
            min_length: v.min_length,
            placeholder: v.placeholder,
            required: v.required,
            style: v.style.try_into()?,
            value: v.value,
        })
    }
}

impl From<TextInput> for TwilightTextInput {
    fn from(v: TextInput) -> Self {
        Self {
            id: v.id,
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
impl TryFrom<TwilightTextInputStyle> for TextInputStyle {
    type Error = anyhow::Error;

    fn try_from(v: TwilightTextInputStyle) -> Result<Self, Self::Error> {
        Ok(match v {
            TwilightTextInputStyle::Short => Self::Short,
            TwilightTextInputStyle::Paragraph => Self::Paragraph,
            other => {
                return Err(UnsupportedComponent(format!("text input style {other:?}")).into())
            }
        })
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

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ISection",
    export_to = "bindings/discord/ISection.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub components: Vec<Component>,
    pub accessory: Box<Component>,
}

use twilight_model::channel::message::component::Section as TwilightSection;
impl TryFrom<TwilightSection> for Section {
    type Error = anyhow::Error;

    fn try_from(v: TwilightSection) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            components: convert_components_lossy(v.components)?,
            accessory: Box::new((*v.accessory).try_into()?),
        })
    }
}

impl TryFrom<Section> for TwilightSection {
    type Error = anyhow::Error;

    fn try_from(v: Section) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            components: v
                .components
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            accessory: Box::new((*v.accessory).try_into()?),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ITextDisplay",
    export_to = "bindings/discord/ITextDisplay.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct TextDisplay {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub content: String,
}

use twilight_model::channel::message::component::TextDisplay as TwilightTextDisplay;
impl From<TwilightTextDisplay> for TextDisplay {
    fn from(v: TwilightTextDisplay) -> Self {
        Self {
            id: v.id,
            content: v.content,
        }
    }
}

impl From<TextDisplay> for TwilightTextDisplay {
    fn from(v: TextDisplay) -> Self {
        Self {
            id: v.id,
            content: v.content,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IThumbnail",
    export_to = "bindings/discord/IThumbnail.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub media: UnfurledMediaItem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spoiler: Option<bool>,

}

use twilight_model::channel::message::component::Thumbnail as TwilightThumbnail;
impl From<TwilightThumbnail> for Thumbnail {
    fn from(v: TwilightThumbnail) -> Self {
        Self {
            id: v.id,
            media: v.media.into(),
            description: v.description,
            spoiler: v.spoiler
        }
    }
}

impl From<Thumbnail> for TwilightThumbnail {
    fn from(v: Thumbnail) -> Self {
        Self {
            id: v.id,
            media: v.media.into(),
            description: v.description,
            spoiler: v.spoiler,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IUnfurledMediaItem",
    export_to = "bindings/discord/IUnfurledMediaItem.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct UnfurledMediaItem {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub proxy_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub content_type: Option<String>,
}

use twilight_model::channel::message::component::UnfurledMediaItem as TwilightUnfurledMediaItem;
impl From<TwilightUnfurledMediaItem> for UnfurledMediaItem {
    fn from(v: TwilightUnfurledMediaItem) -> Self {
        Self {
            url: v.url,
            proxy_url: v.proxy_url,
            height: v.height.flatten(),
            width: v.width.flatten(),
            content_type: v.content_type,
        }
    }
}

impl From<UnfurledMediaItem> for TwilightUnfurledMediaItem {
    fn from(v: UnfurledMediaItem) -> Self {
        Self {
            url: v.url,
            proxy_url: v.proxy_url,
            height: Some(v.height),
            width: Some(v.width),
            content_type: v.content_type,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IMediaGallery",
    export_to = "bindings/discord/IMediaGallery.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct MediaGallery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub items: Vec<MediaGalleryItem>,
}

use twilight_model::channel::message::component::MediaGallery as TwilightMediaGallery;
impl TryFrom<TwilightMediaGallery> for MediaGallery {
    type Error = anyhow::Error;

    fn try_from(v: TwilightMediaGallery) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            items: v
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<MediaGallery> for TwilightMediaGallery {
    type Error = anyhow::Error;

    fn try_from(v: MediaGallery) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            items: v
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IMediaGalleryItem",
    export_to = "bindings/discord/IMediaGalleryItem.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct MediaGalleryItem {
    pub media: UnfurledMediaItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spoiler: Option<bool>,
}

use twilight_model::channel::message::component::MediaGalleryItem as TwilightMediaGalleryItem;
impl From<TwilightMediaGalleryItem> for MediaGalleryItem {
    fn from(v: TwilightMediaGalleryItem) -> Self {
        Self {
            media: v.media.into(),
            description: v.description,
            spoiler: v.spoiler,
        }
    }
}

impl From<MediaGalleryItem> for TwilightMediaGalleryItem {
    fn from(v: MediaGalleryItem) -> Self {
        Self {
            media: v.media.into(),
            description: v.description,
            spoiler: v.spoiler,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IFile",
    export_to = "bindings/discord/IFile.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct FileDisplay {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub file: UnfurledMediaItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spoiler: Option<bool>,
}

use twilight_model::channel::message::component::FileDisplay as TwilightFileDisplay;
impl From<TwilightFileDisplay> for FileDisplay {
    fn from(v: TwilightFileDisplay) -> Self {
        Self {
            id: v.id,
            file: v.file.into(),
            spoiler: v.spoiler,
        }
    }
}

impl From<FileDisplay> for TwilightFileDisplay {
    fn from(v: FileDisplay) -> Self {
        Self {
            id: v.id,
            file: v.file.into(),
            spoiler: v.spoiler,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ISeparator",
    export_to = "bindings/discord/ISeparator.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct Separator {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub divider: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spacing: Option<SeparatorSpacingSize>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/SeparatorSpacingSize.ts")]
pub enum SeparatorSpacingSize {
    Small,
    Large,
}

use twilight_model::channel::message::component::Separator as TwilightSeparator;
impl TryFrom<TwilightSeparator> for Separator {
    type Error = anyhow::Error;

    fn try_from(v: TwilightSeparator) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            divider: v.divider,
            spacing: v.spacing.map(TryInto::try_into).transpose()?,
        })
    }
}

impl From<Separator> for TwilightSeparator {
    fn from(v: Separator) -> Self {
        Self {
            id: v.id,
            divider: v.divider,
            spacing: v.spacing.map(Into::into),
        }
    }
}

use twilight_model::channel::message::component::SeparatorSpacingSize as TwilightSeparatorSpacingSize;
impl TryFrom<TwilightSeparatorSpacingSize> for SeparatorSpacingSize {
    type Error = anyhow::Error;

    fn try_from(v: TwilightSeparatorSpacingSize) -> Result<Self, Self::Error> {
        Ok(match v {
            TwilightSeparatorSpacingSize::Small => Self::Small,
            TwilightSeparatorSpacingSize::Large => Self::Large,
            other => {
                return Err(
                    UnsupportedComponent(format!("separator spacing {other:?}")).into(),
                )
            }
        })
    }
}

impl From<SeparatorSpacingSize> for TwilightSeparatorSpacingSize {
    fn from(v: SeparatorSpacingSize) -> Self {
        match v {
            SeparatorSpacingSize::Small => Self::Small,
            SeparatorSpacingSize::Large => Self::Large,
        }
    }
}


#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IContainer",
    export_to = "bindings/discord/IContainer.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub accent_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spoiler: Option<bool>,
    pub components: Vec<Component>,
}

use twilight_model::channel::message::component::Container as TwilightContainer;
impl TryFrom<TwilightContainer> for Container {
    type Error = anyhow::Error;

    fn try_from(v: TwilightContainer) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            accent_color: v.accent_color.flatten(),
            spoiler: v.spoiler,
            components: convert_components_lossy(v.components)?,
        })
    }
}

impl TryFrom<Container> for TwilightContainer {
    type Error = anyhow::Error;

    fn try_from(v: Container) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            accent_color: Some(v.accent_color),
            spoiler: v.spoiler,
            components: v
                .components
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ILabel",
    export_to = "bindings/discord/ILabel.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub component: Box<Component>,
}

use twilight_model::channel::message::component::Label as TwilightLabel;
impl TryFrom<TwilightLabel> for Label {
    type Error = anyhow::Error;

    fn try_from(v: TwilightLabel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            label: v.label,
            description: v.description,
            component: Box::new((*v.component).try_into()?)
        })
    }
}

impl TryFrom<Label> for TwilightLabel {
    type Error = anyhow::Error;

    fn try_from(v: Label) -> Result<Self, Self::Error> {
        Ok(Self {
            id: v.id,
            label: v.label,
            description: v.description,
            component: Box::new((*v.component).try_into()?)
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "IFileUpload",
    export_to = "bindings/discord/IFileUpload.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub custom_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_values: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_values: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub required: Option<bool>,
}

use twilight_model::channel::message::component::FileUpload as TwilightFileUpload;
impl From<TwilightFileUpload> for FileUpload {
    fn from(v: TwilightFileUpload) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            max_values: v.max_values,
            min_values: v.min_values,
            required: v.required,
        }
    }
}

impl From<FileUpload> for TwilightFileUpload {
    fn from(v: FileUpload) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            max_values: v.max_values,
            min_values: v.min_values,
            required: v.required,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ICheckbox",
    export_to = "bindings/discord/ICheckbox.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct Checkbox {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub custom_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default: Option<bool>,
}

use twilight_model::channel::message::component::Checkbox as TwilightCheckbox;
impl From<TwilightCheckbox> for Checkbox {
    fn from(v: TwilightCheckbox) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            default: v.default,
        }
    }
}

impl From<Checkbox> for TwilightCheckbox {
    fn from(v: Checkbox) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            default: v.default,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ICheckboxGroup",
    export_to = "bindings/discord/ICheckboxGroup.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct CheckboxGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<i32>,
    pub custom_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_values: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_values: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub required: Option<bool>,

    pub options: Vec<CheckboxGroupOption>,
}

use twilight_model::channel::message::component::CheckboxGroup as TwilightCheckboxGroup;
impl From<TwilightCheckboxGroup> for CheckboxGroup {
    fn from(v: TwilightCheckboxGroup) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            max_values: v.max_values,
            min_values: v.min_values,
            required: v.required,
            options: v
                .options
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<CheckboxGroup> for TwilightCheckboxGroup {
    fn from(v: CheckboxGroup) -> Self {
        Self {
            id: v.id,
            custom_id: v.custom_id,
            max_values: v.max_values,
            min_values: v.min_values,
            required: v.required,
            options: v
                .options
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}


#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(
    export,
    rename = "ICheckboxGroupOption",
    export_to = "bindings/discord/ICheckboxGroupOption.ts"
)]
#[serde(rename_all = "camelCase")]
pub struct CheckboxGroupOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub label: String,
    pub value: String,
}

use twilight_model::channel::message::component::CheckboxGroupOption as TwilightCheckboxGroupOption;
impl From<TwilightCheckboxGroupOption> for CheckboxGroupOption {
    fn from (v: TwilightCheckboxGroupOption) -> Self {
        Self {
            default: v.default,
            description: v.description,
            label: v.label,
            value: v.value,
        }
    }
}

impl From<CheckboxGroupOption> for TwilightCheckboxGroupOption {
    fn from (v: CheckboxGroupOption) -> Self {
        Self {
            default: v.default,
            description: v.description,
            label: v.label,
            value: v.value,
        }
    }
}
