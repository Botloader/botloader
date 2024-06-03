use serde::Deserialize;
use ts_rs::TS;
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/image/SupportedImageFormats.ts")]
pub enum SupportedImageFormats {
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

pub enum ImageFormat {
    Encoded(image::ImageFormat),
    Decoded(image::ColorType),
}
