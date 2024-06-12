use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/image/SupportedImageFormat.ts")]
#[serde(rename_all = "lowercase")]
pub enum SupportedImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Pnm,
    Tiff,
    Tga,
    Bmp,
    Ico,
    Hdr,
    OpenExr,
    Qoi,
    Avif,
}

impl TryFrom<image::ImageFormat> for SupportedImageFormat {
    type Error = ();

    fn try_from(value: image::ImageFormat) -> Result<Self, Self::Error> {
        match value {
            image::ImageFormat::Png => Ok(Self::Png),
            image::ImageFormat::Jpeg => Ok(Self::Jpeg),
            image::ImageFormat::Gif => Ok(Self::Gif),
            image::ImageFormat::WebP => Ok(Self::WebP),
            image::ImageFormat::Pnm => Ok(Self::Pnm),
            image::ImageFormat::Tiff => Ok(Self::Tiff),
            image::ImageFormat::Tga => Ok(Self::Tga),
            image::ImageFormat::Bmp => Ok(Self::Bmp),
            image::ImageFormat::Ico => Ok(Self::Ico),
            image::ImageFormat::Hdr => Ok(Self::Hdr),
            image::ImageFormat::OpenExr => Ok(Self::OpenExr),
            image::ImageFormat::Avif => Ok(Self::Avif),
            image::ImageFormat::Qoi => Ok(Self::Qoi),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/image/SupportedEncodeImageFormat.ts")]
#[serde(rename_all = "lowercase")]
pub enum SupportedEncodeImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/internal/ImageProperties.ts")]
pub struct ImageProperties {
    #[serde(rename = "formatName")]
    pub format: SupportedImageFormat,
    pub width: u32,
    pub height: u32,
}
