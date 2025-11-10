use std::path::PathBuf;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use clap::{Parser, arg};
use dotenvy::{dotenv_override, var};
use entity::user_data::Model;
use entity::{user_data, user_properties};
use sea_orm::Database;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::file_user::file::load_user;
use crate::web::api::Api;

mod file_user;
mod instance_handling;
mod web;

type GenResult<T> = Result<T, GenError>;
type GenError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    password: Option<String>,
    #[arg(short, long)]
    user: Option<String>,
}

#[tokio::main]
async fn main() -> GenResult<()> {
    dotenv_override().unwrap();
    let args = Args::parse();
    if let Some(password) = args.password {
        println!("{}", encode_password(password));
    } else if let Some(path) = args.user {
        let db = Database::connect(&var("DATABASE_URL").unwrap())
            .await
            .expect("Could not connect to database");
        let data = load_user(PathBuf::from(path));
        let id = add_user_to_db(&db, data.0, data.1).await.unwrap();
        println!("added user with ID of {}", id.user_data_id);
    }
    Api::new().await?.serve().await?;
    Ok(())
}

fn encode_password(password: String) -> String {
    let secret = var("PASSWORD_SECRET").expect("No password secret set");
    BASE64_STANDARD_NO_PAD.encode(
        simplestcrypt::encrypt_and_serialize(secret.as_bytes(), password.as_bytes()).unwrap(),
    )
}

async fn add_user_to_db(
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
