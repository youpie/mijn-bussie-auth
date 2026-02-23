use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use dotenvy::var;
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;

use crate::error::OptionResult;
use crate::web::api::Api;

pub mod error;
mod instance_handling;
mod web;
pub mod bypass;

type GenResult<T> = Result<T, GenError>;
type GenError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[dotenvy::load(override_ = true)]
#[tokio::main]
async fn main() -> GenResult<()> {
    CryptoProvider::install_default(default_provider()).unwrap();
    Api::new().await?.serve().await?;
    Ok(())
}

pub fn encrypt_value(value: &str) -> GenResult<String> {
    let secret_string = var("PASSWORD_SECRET")?;
    let secret = secret_string.as_bytes();
    let value = BASE64_STANDARD_NO_PAD.encode(
        simplestcrypt::encrypt_and_serialize(secret, value.as_bytes())
            .ok()
            .result_reason("Failed to encode password")?,
    );
    Ok(value)
}

fn decrypt_value(encrypted_value: &str, make_lowercase: bool) -> GenResult<String> {
    let secret_string = var("PASSWORD_SECRET")?;
    let secret = secret_string.as_bytes();
    let mut text = String::from_utf8(
        simplestcrypt::deserialize_and_decrypt(
            secret,
            &BASE64_STANDARD_NO_PAD.decode(encrypted_value)?,
        )
        .ok()
        .result_reason("Could not deserialize password")?,
    )?;
    if make_lowercase {
        text = text.to_lowercase();
    }
    Ok(text)
}

pub trait Client {
    /// Censor sensitive data
    ///
    /// You need to select which attribute you want to be kept
    ///
    /// Zero-trust
    fn censor(self) -> Self;
}
