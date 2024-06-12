use deno_core::{error::AnyError, op2, ToJsBuffer};

deno_core::extension!(bl_base64, ops = [op_base64_decode, op_base64_encode]);

#[op2]
#[serde]
fn op_base64_decode(#[string] input: String) -> Result<ToJsBuffer, AnyError> {
    let mut s = input.into_bytes();
    let decoded_len = forgiving_base64_decode_inplace(&mut s)?;
    s.truncate(decoded_len);
    Ok(s.into())
}

/// See <https://infra.spec.whatwg.org/#forgiving-base64>
#[inline]
fn forgiving_base64_decode_inplace(input: &mut [u8]) -> Result<usize, AnyError> {
    let decoded = base64_simd::forgiving_decode_inplace(input)
        .map_err(|_| anyhow::anyhow!("Failed to decode base64"))?;
    Ok(decoded.len())
}

#[op2]
#[string]
fn op_base64_encode(#[buffer] s: &[u8]) -> String {
    forgiving_base64_encode(s)
}

/// See <https://infra.spec.whatwg.org/#forgiving-base64>
#[inline]
fn forgiving_base64_encode(s: &[u8]) -> String {
    base64_simd::STANDARD.encode_to_string(s)
}
