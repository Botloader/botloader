use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::message::ReactionType;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/ComponentType.ts")]
pub enum ComponentType {
    ActionRow,
    Button,
    SelectMenu,
}

use twilight_model::application::component::ComponentType as TwilightComponentType;
impl From<TwilightComponentType> for ComponentType {
    fn from(v: TwilightComponentType) -> Self {
        match v {
            TwilightComponentType::ActionRow => Self::ActionRow,
            TwilightComponentType::Button => Self::Button,
            TwilightComponentType::SelectMenu => Self::SelectMenu,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/Component.ts")]
// #[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
}

use twilight_model::application::component::Component as TwilightComponent;
impl From<TwilightComponent> for Component {
    fn from(v: TwilightComponent) -> Self {
        match v {
            TwilightComponent::ActionRow(inner) => Self::ActionRow(inner.into()),
            TwilightComponent::Button(inner) => Self::Button(inner.into()),
            TwilightComponent::SelectMenu(inner) => Self::SelectMenu(inner.into()),
        }
    }
}
impl From<Component> for TwilightComponent {
    fn from(v: Component) -> Self {
        match v {
            Component::ActionRow(inner) => Self::ActionRow(inner.into()),
            Component::Button(inner) => Self::Button(inner.into()),
            Component::SelectMenu(inner) => Self::SelectMenu(inner.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/ActionRow.ts")]
#[serde(rename_all = "camelCase")]
pub struct ActionRow {
    pub components: Vec<Component>,
}

use twilight_model::application::component::ActionRow as TwilightActionRow;
impl From<TwilightActionRow> for ActionRow {
    fn from(v: TwilightActionRow) -> Self {
        Self {
            components: v.components.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<ActionRow> for TwilightActionRow {
    fn from(v: ActionRow) -> Self {
        Self {
            components: v.components.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/Button.ts")]
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

use twilight_model::application::component::Button as TwilightButton;
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
#[ts(export, export_to = "bindings/discord/SelectMenu.ts")]
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
}

use twilight_model::application::component::SelectMenu as TwilightSelectMenu;
impl From<TwilightSelectMenu> for SelectMenu {
    fn from(v: TwilightSelectMenu) -> Self {
        Self {
            custom_id: v.custom_id,
            disabled: v.disabled,
            min_values: v.min_values,
            max_values: v.max_values,
            options: v.options.into_iter().map(Into::into).collect(),
            placeholder: v.placeholder,
        }
    }
}
impl From<SelectMenu> for TwilightSelectMenu {
    fn from(v: SelectMenu) -> Self {
        Self {
            custom_id: v.custom_id,
            disabled: v.disabled,
            min_values: v.min_values,
            max_values: v.max_values,
            options: v.options.into_iter().map(Into::into).collect(),
            placeholder: v.placeholder,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/discord/SelectMenuOption.ts")]
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

use twilight_model::application::component::select_menu::SelectMenuOption as TwilightSelectMenuOption;
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
}

use twilight_model::application::component::button::ButtonStyle as TwilightButtonStyle;
impl From<TwilightButtonStyle> for ButtonStyle {
    fn from(v: TwilightButtonStyle) -> Self {
        match v {
            TwilightButtonStyle::Primary => Self::Primary,
            TwilightButtonStyle::Secondary => Self::Secondary,
            TwilightButtonStyle::Success => Self::Success,
            TwilightButtonStyle::Danger => Self::Danger,
            TwilightButtonStyle::Link => Self::Link,
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
        }
    }
}
