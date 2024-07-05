use std::io::Cursor;

use anyhow::anyhow;
use deno_core::{error::AnyError, op2};
use image::{
    codecs::{
        gif::{GifDecoder, GifEncoder},
        jpeg::JpegEncoder,
        png::{PngDecoder, PngEncoder},
        webp::{WebPDecoder, WebPEncoder},
    },
    imageops, AnimationDecoder, ImageDecoder,
};
use runtime_models::image::{ImageProperties, SupportedEncodeImageFormat, SupportedImageFormat};

const MAX_IMAGE_BYTES: u64 = 10_000_000;

deno_core::extension!(
    bl_image,
    ops = [op_bl_image_properties, op_bl_image_transcode],
);

#[op2]
#[serde]
fn op_bl_image_properties(#[arraybuffer] input: &[u8]) -> Result<ImageProperties, AnyError> {
    let buf = Cursor::new(&input);
    let reader = image::ImageReader::new(buf).with_guessed_format()?;

    let Some(format) = reader.format() else {
        // Don't think this is reachable as the above will return an error if it fails
        return Err(anyhow!("could not guess image format"));
    };

    let translated_format = SupportedImageFormat::try_from(format)
        .map_err(|_| anyhow!("Unsupported format {:?}", format))?;

    let dimensions = reader.into_dimensions()?;

    Ok(ImageProperties {
        format: translated_format,
        width: dimensions.0,
        height: dimensions.1,
    })
}

#[op2]
#[buffer]
fn op_bl_image_transcode(
    #[arraybuffer] input: &[u8],
    #[serde] in_format: SupportedImageFormat,
    #[serde] out_format: SupportedEncodeImageFormat,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Vec<u8>, AnyError> {
    let resize = match (width, height) {
        (Some(width), Some(height)) => Some(Dimensions { width, height }),
        (None, None) => None,
        _ => {
            return Err(anyhow!("Both width and height need to specified to resize"));
        }
    };

    tracing::info!("transcoding input {:?} output {:?}", in_format, out_format);

    let new_image_data = match in_format {
        SupportedImageFormat::Gif => transcode_animation(
            input,
            InputAnimationFormat::Gif,
            out_format.try_into()?,
            resize,
        )?,
        SupportedImageFormat::Png => {
            let mut buf = Cursor::new(&input);
            let png_decoder = PngDecoder::new(&mut buf)?;
            if png_decoder.is_apng()? {
                transcode_animation(
                    input,
                    InputAnimationFormat::APng,
                    out_format.try_into()?,
                    resize,
                )?
            } else {
                transcode_image(input, out_format, resize)?
            }
        }
        SupportedImageFormat::WebP => {
            let mut buf = Cursor::new(&input);
            let png_decoder = WebPDecoder::new(&mut buf)?;
            if png_decoder.has_animation() {
                transcode_animation(
                    input,
                    InputAnimationFormat::Webp,
                    out_format.try_into()?,
                    resize,
                )?
            } else {
                transcode_image(input, out_format, resize)?
            }
        }
        _ => transcode_image(input, out_format, resize)?,
    };

    Ok(new_image_data)
}

fn limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(5000);
    limits.max_image_height = Some(5000);
    limits.max_alloc = Some(MAX_IMAGE_BYTES);

    limits
}

struct Dimensions {
    width: u32,
    height: u32,
}

fn transcode_image(
    input: &[u8],
    format: SupportedEncodeImageFormat,
    resize: Option<Dimensions>,
) -> Result<Vec<u8>, AnyError> {
    let buf = Cursor::new(&input);
    let mut reader = image::ImageReader::new(buf).with_guessed_format()?;
    reader.limits(limits());
    let mut loaded = reader.decode()?;

    if let Some(resize) = resize {
        loaded = loaded.resize_exact(
            resize.width,
            resize.height,
            image::imageops::FilterType::CatmullRom,
        );
    }

    let mut output = Vec::<u8>::new();
    match format {
        SupportedEncodeImageFormat::Jpeg => {
            let encoder = JpegEncoder::new(&mut output);
            loaded.write_with_encoder(encoder)?;
        }
        SupportedEncodeImageFormat::Png => {
            let encoder = PngEncoder::new(&mut output);
            loaded.write_with_encoder(encoder)?;
        }
        SupportedEncodeImageFormat::WebP => {
            let encoder = WebPEncoder::new_lossless(&mut output);
            loaded.write_with_encoder(encoder)?;
        }
        SupportedEncodeImageFormat::Gif => {
            let mut encoder = GifEncoder::new(&mut output);
            encoder.encode(
                loaded.as_bytes(),
                loaded.width(),
                loaded.height(),
                loaded.color().into(),
            )?;
        }
    };

    Ok(output)
}

enum InputAnimationFormat {
    Gif,
    APng,
    Webp,
}

enum OutputAnimationFormat {
    Gif,
}

impl TryFrom<SupportedEncodeImageFormat> for OutputAnimationFormat {
    type Error = anyhow::Error;

    fn try_from(value: SupportedEncodeImageFormat) -> Result<Self, Self::Error> {
        match value {
            SupportedEncodeImageFormat::Gif => Ok(Self::Gif),
            SupportedEncodeImageFormat::Png => Err(anyhow!(
                "Encoding animations into this format has not been implemented yet"
            )),
            SupportedEncodeImageFormat::Jpeg => Err(anyhow!("Unsupported format for animations")),
            SupportedEncodeImageFormat::WebP => Err(anyhow!(
                "Encoding animations into this format has not been implemented yet"
            )),
        }
    }
}

fn transcode_animation(
    input: &[u8],
    input_format: InputAnimationFormat,
    _output_format: OutputAnimationFormat,
    resize: Option<Dimensions>,
) -> Result<Vec<u8>, AnyError> {
    let buf = Cursor::new(input);

    let limits = limits();
    let frame_iter = match input_format {
        InputAnimationFormat::Gif => {
            let mut decoder = GifDecoder::new(buf)?;
            decoder.set_limits(limits)?;
            decoder.into_frames()
        }
        InputAnimationFormat::APng => {
            let mut decoder = PngDecoder::new(buf)?;
            decoder.set_limits(limits)?;
            let decoder = decoder.apng()?;
            decoder.into_frames()
        }
        InputAnimationFormat::Webp => {
            let mut decoder = WebPDecoder::new(buf)?;
            decoder.set_limits(limits)?;
            decoder.into_frames()
        }
    };

    let mut output = Vec::<u8>::new();

    let mut encoder = GifEncoder::new(&mut output);
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    for frame in frame_iter {
        let frame = match frame {
            Ok(v) => v,
            Err(err) => {
                return Err(anyhow!("failed decoding animation frame {}", err));
            }
        };

        let left = frame.left();
        let top = frame.top();
        let delay = frame.delay();

        let buf = frame.into_buffer();
        let raw = if let Some(resize) = &resize {
            // let resized: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            imageops::resize(
                &buf,
                resize.width,
                resize.height,
                imageops::FilterType::CatmullRom,
            )
        } else {
            buf
        };

        encoder.encode_frame(image::Frame::from_parts(raw, left, top, delay))?;
    }
    drop(encoder);

    Ok(output)
}
