use rand::{thread_rng, Rng};

pub fn gen_token() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| thread_rng().gen::<u8>()).collect();
    base64::encode_config(random_bytes, base64::URL_SAFE_NO_PAD)
}
