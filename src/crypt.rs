use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use dotenvy::var;

use crate::prelude::*;

pub fn encrypt_value(value: &str) -> GenResult<String> {
    let secret_string = var("PASSWORD_SECRET").d()?;
    let secret = secret_string.as_bytes();
    let value = BASE64_STANDARD_NO_PAD.encode(
        simplestcrypt::encrypt_and_serialize(secret, value.as_bytes())
            .ok()
            .result_reason("Failed to encode password")?,
    );
    Ok(value)
}

fn decrypt_value(encrypted_value: &str, make_lowercase: bool) -> GenResult<String> {
    let secret_string = var("PASSWORD_SECRET").d()?;
    let secret = secret_string.as_bytes();
    let mut text = String::from_utf8(
        simplestcrypt::deserialize_and_decrypt(
            secret,
            &BASE64_STANDARD_NO_PAD.decode(encrypted_value).d()?,
        )
        .ok()
        .result_reason("Could not deserialize password")?,
    )
    .d()?;
    if make_lowercase {
        text = text.to_lowercase();
    }
    Ok(text)
}
