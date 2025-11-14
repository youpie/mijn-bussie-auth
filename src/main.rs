use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use dotenvy::{dotenv_override, var};
use entity::user_data::Model;
use entity::{user_data, user_properties};
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::web::api::Api;

mod file_user;
mod instance_handling;
mod web;

type GenResult<T> = Result<T, GenError>;
type GenError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> GenResult<()> {
    dotenv_override().unwrap();
    CryptoProvider::install_default(default_provider()).unwrap();
    Api::new().await?.serve().await?;
    Ok(())
}

fn encode_password(password: String) -> String {
    let secret = var("PASSWORD_SECRET").expect("No password secret set");
    BASE64_STANDARD_NO_PAD.encode(
        simplestcrypt::encrypt_and_serialize(secret.as_bytes(), password.as_bytes()).unwrap(),
    )
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

async fn update_user_in_db(
    db: &DatabaseConnection,
    user_properties: user_properties::ActiveModel,
    user_data: user_data::ActiveModel,
) -> GenResult<Model> {
    user_properties::Entity::update(user_properties)
        .exec(db)
        .await?;

    let data_res = user_data::Entity::update(user_data).exec(db).await?;
    Ok(data_res)
}
