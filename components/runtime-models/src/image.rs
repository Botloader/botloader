use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/image/SupportedImageFormat.ts")]
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

    RawL8,
    RawLa8,
    RawRgb8,
    RawRgba8,
    RawL16,
    RawLa16,
    RawRgb16,
    RawRgba16,
    RawRgb32F,
    RawRgba32F,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/image/SupportedEncodeImageFormat.ts")]
pub enum SupportedEncodeImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/image/ImageProperties.ts")]
pub struct ImageProperties {
    pub format: SupportedEncodeImageFormat,
    pub width: u32,
    pub height: u32,
}
