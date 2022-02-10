use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::datetime::Timestamp;

use crate::util::NotBigU64;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/Embed.ts")]
pub struct Embed {
    #[serde(default)]
    #[ts(optional)]
    pub author: Option<EmbedAuthor>,

    #[serde(default)]
    #[ts(optional)]
    pub color: Option<u32>,

    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub fields: Option<Vec<EmbedField>>,

    #[serde(default)]
    #[ts(optional)]
    pub footer: Option<EmbedFooter>,

    #[serde(default)]
    #[ts(optional)]
    pub image: Option<EmbedImage>,

    #[serde(default)]
    #[ts(optional)]
    pub kind: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub provider: Option<EmbedProvider>,

    #[serde(default)]
    #[ts(optional)]
    pub thumbnail: Option<EmbedThumbnail>,

    #[serde(default)]
    #[ts(optional)]
    pub timestamp: Option<NotBigU64>,

    #[serde(default)]
    #[ts(optional)]
    pub title: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,

    #[serde(default)]
    #[ts(optional)]
    pub video: Option<EmbedVideo>,
}
impl From<Embed> for twilight_model::channel::embed::Embed {
    fn from(v: Embed) -> Self {
        Self {
            author: v.author.map(From::from),
            color: v.color,
            description: v.description,
            fields: v
                .fields
                .unwrap_or_default()
                .into_iter()
                .map(From::from)
                .collect(),
            footer: v.footer.map(From::from),
            image: v.image.map(From::from),
            kind: v.kind.unwrap_or_else(|| "rich".to_string()),
            provider: v.provider.map(From::from),
            thumbnail: v.thumbnail.map(From::from),
            timestamp: v
                .timestamp
                .map(|v| Timestamp::from_micros(v.0 as i64 * 1000).ok())
                .flatten(),
            title: v.title,
            url: v.url,
            video: v.video.map(From::from),
        }
    }
}

impl From<twilight_model::channel::embed::Embed> for Embed {
    fn from(v: twilight_model::channel::embed::Embed) -> Self {
        Self {
            author: v.author.map(From::from),
            color: v.color,
            description: v.description,
            fields: if v.fields.is_empty() {
                None
            } else {
                Some(v.fields.into_iter().map(From::from).collect())
            },
            footer: v.footer.map(From::from),
            image: v.image.map(From::from),
            kind: if v.kind.is_empty() {
                None
            } else {
                Some(v.kind)
            },
            provider: v.provider.map(From::from),
            thumbnail: v.thumbnail.map(From::from),
            timestamp: v
                .timestamp
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            title: v.title,
            url: v.url,
            video: v.video.map(From::from),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedAuthor.ts")]
pub struct EmbedAuthor {
    pub name: String,

    #[serde(default)]
    #[ts(optional)]
    pub icon_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub proxy_icon_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
}
impl From<EmbedAuthor> for twilight_model::channel::embed::EmbedAuthor {
    fn from(v: EmbedAuthor) -> Self {
        Self {
            icon_url: v.icon_url,
            name: v.name,
            proxy_icon_url: v.proxy_icon_url,
            url: v.url,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedAuthor> for EmbedAuthor {
    fn from(v: twilight_model::channel::embed::EmbedAuthor) -> Self {
        Self {
            icon_url: v.icon_url,
            name: v.name,
            proxy_icon_url: v.proxy_icon_url,
            url: v.url,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedField.ts")]
pub struct EmbedField {
    #[serde(default)]
    #[ts(optional)]
    pub inline: Option<bool>,
    pub name: String,
    pub value: String,
}
impl From<EmbedField> for twilight_model::channel::embed::EmbedField {
    fn from(v: EmbedField) -> Self {
        Self {
            inline: v.inline.unwrap_or_default(),
            name: v.name,
            value: v.value,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedField> for EmbedField {
    fn from(v: twilight_model::channel::embed::EmbedField) -> Self {
        Self {
            inline: Some(v.inline),
            name: v.name,
            value: v.value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedFooter.ts")]
pub struct EmbedFooter {
    pub text: String,

    #[serde(default)]
    #[ts(optional)]
    pub icon_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub proxy_icon_url: Option<String>,
}

impl From<EmbedFooter> for twilight_model::channel::embed::EmbedFooter {
    fn from(v: EmbedFooter) -> Self {
        Self {
            icon_url: v.icon_url,
            proxy_icon_url: v.proxy_icon_url,
            text: v.text,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedFooter> for EmbedFooter {
    fn from(v: twilight_model::channel::embed::EmbedFooter) -> Self {
        Self {
            icon_url: v.icon_url,
            proxy_icon_url: v.proxy_icon_url,
            text: v.text,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedImage.ts")]
pub struct EmbedImage {
    pub url: String,

    #[serde(default)]
    #[ts(optional)]
    pub height: Option<i32>,
    #[serde(default)]
    #[ts(optional)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub width: Option<i32>,
}
impl From<EmbedImage> for twilight_model::channel::embed::EmbedImage {
    fn from(v: EmbedImage) -> Self {
        Self {
            height: v.height.map(|v| v as u64),
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width.map(|v| v as u64),
        }
    }
}

impl From<twilight_model::channel::embed::EmbedImage> for EmbedImage {
    fn from(v: twilight_model::channel::embed::EmbedImage) -> Self {
        Self {
            height: v.height.map(|v| v as i32),
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width.map(|v| v as i32),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedProvider.ts")]
pub struct EmbedProvider {
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
}

impl From<EmbedProvider> for twilight_model::channel::embed::EmbedProvider {
    fn from(v: EmbedProvider) -> Self {
        Self {
            name: v.name,
            url: v.url,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedProvider> for EmbedProvider {
    fn from(v: twilight_model::channel::embed::EmbedProvider) -> Self {
        Self {
            name: v.name,
            url: v.url,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedThumbnail.ts")]
pub struct EmbedThumbnail {
    pub url: String,

    #[serde(default)]
    #[ts(optional)]
    pub height: Option<i32>,
    #[serde(default)]
    #[ts(optional)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub width: Option<i32>,
}

impl From<EmbedThumbnail> for twilight_model::channel::embed::EmbedThumbnail {
    fn from(v: EmbedThumbnail) -> Self {
        Self {
            height: v.height.map(|v| v as u64),
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width.map(|v| v as u64),
        }
    }
}
impl From<twilight_model::channel::embed::EmbedThumbnail> for EmbedThumbnail {
    fn from(v: twilight_model::channel::embed::EmbedThumbnail) -> Self {
        Self {
            height: v.height.map(|v| v as i32),
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width.map(|v| v as i32),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/EmbedVideo.ts")]
pub struct EmbedVideo {
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub height: Option<i32>,
    #[serde(default)]
    #[ts(optional)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub width: Option<i32>,
}

impl From<twilight_model::channel::embed::EmbedVideo> for EmbedVideo {
    fn from(v: twilight_model::channel::embed::EmbedVideo) -> Self {
        Self {
            height: v.height.map(|v| v as i32),
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width.map(|v| v as i32),
        }
    }
}

impl From<EmbedVideo> for twilight_model::channel::embed::EmbedVideo {
    fn from(v: EmbedVideo) -> Self {
        Self {
            height: v.height.map(|v| v as u64),
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width.map(|v| v as u64),
        }
    }
}
