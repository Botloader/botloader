use anyhow::anyhow;
use deno_core::{error::AnyError, op2};
use runtime_models::image::{ImageProperties, SupportedImageFormat};

struct ImageState {
    pending_image_data: Option<Vec<u8>>,
}

deno_core::extension!(
    bl_image,
    ops = [op_image_properties],
    state = |state| {
        state.put(ImageState {
            pending_image_data: None,
        });
    },
);

#[op2]
#[serde]
fn op_image_properties(#[arraybuffer] input: &[u8]) -> Result<ImageProperties, AnyError> {
    todo!()
}

#[op2]
#[serde]
fn op_image_resize(
    #[arraybuffer] input: &[u8],
    #[serde] in_format: SupportedImageFormat,
    #[state] state: &mut ImageState,
    width: u32,
    height: u32,
) -> Result<ImageProperties, AnyError> {
    state.pending_image_data = Some(Vec::new());
    todo!()
}

#[op2]
#[serde]
fn op_image_transcode(
    #[arraybuffer] input: &[u8],
    #[serde] in_format: SupportedImageFormat,
    #[state] state: &mut ImageState,
    width: u32,
    height: u32,
) -> Result<ImageProperties, AnyError> {
    state.pending_image_data = Some(Vec::new());
    todo!()
}

#[op2]
#[buffer]
fn op_last_image_data(#[state] state: &mut ImageState) -> Result<Vec<u8>, AnyError> {
    let data = state.pending_image_data.take();
    if let Some(data) = data {
        Ok(data)
    } else {
        Err(anyhow!("No last image data"))
    }
}
