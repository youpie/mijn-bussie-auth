use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use dotenvy::var;
use entity::user_data::Model;
use entity::{user_data, user_properties};
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::error::OptionResult;
use crate::web::api::Api;

pub mod error;
mod instance_handling;
mod web;

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

async fn add_new_user_to_db(
    db: &DatabaseConnection,
    user_properties: user_properties::ActiveModel,
    mut user_data: user_data::ActiveModel,
) -> GenResult<Model> {
    let res = user_properties::Entity::insert(user_properties)
        .exec(db)
        .await?;
    println!("id {}", res.last_insert_id);
    user_data.user_properties = Set(res.last_insert_id);

    let data_res = user_data::Entity::insert(user_data)
        .exec_with_returning(db)
        .await?;
    Ok(data_res)
}
