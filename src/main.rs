use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;

use crate::prelude::*;

mod crypt;
mod error;
pub mod instance_handling;
pub mod prelude;
mod web;

#[dotenvy::load(override_ = true)]
#[tokio::main]
async fn main() -> GenResult<()> {
    tracing_subscriber::fmt::init();
    CryptoProvider::install_default(default_provider()).unwrap();
    AppState::new().await?.serve().await?;
    Ok(())
}

pub trait Client {
    /// Censor sensitive data
    ///
    /// You need to select which attribute you want to be kept
    ///
    /// Zero-trust
    fn censor(self) -> Self;
}
