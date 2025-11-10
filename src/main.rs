use std::path::PathBuf;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use bcrypt::DEFAULT_COST;
use clap::{Parser, arg};
use dotenvy::{dotenv_override, var};
use entity::{user_data, user_properties};
use sea_orm::Database;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::file_user::file::load_user;

mod file_user;
mod web;

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
async fn main() {
    dotenv_override().unwrap();
    let secret = var("PASSWORD_SECRET").unwrap();
    let args = Args::parse();
    if let Some(password) = args.password {
        println!("{}", encode_password(password, secret));
    } else if let Some(path) = args.user {
        let db = Database::connect(&var("DATABASE_URL").unwrap())
            .await
            .expect("Could not connect to database");
        let data = load_user(PathBuf::from(path), secret);
        let id = add_user_to_db(&db, data).await;
        println!("added user with ID of {id}");
    }
}

fn encode_password(password: String, secret: String) -> String {
    let secret = secret.as_bytes();
    BASE64_STANDARD_NO_PAD
        .encode(simplestcrypt::encrypt_and_serialize(secret, password.as_bytes()).unwrap())
}

async fn add_user_to_db(
    db: &DatabaseConnection,
    user: (user_properties::ActiveModel, user_data::ActiveModel),
) -> i32 {
    let res = user_properties::Entity::insert(user.0)
        .exec(db)
        .await
        .unwrap();
    let mut data = user.1;
    println!("id {}", res.last_insert_id);
    data.user_properties = Set(res.last_insert_id);

    let data_res = user_data::Entity::insert(data).exec(db).await.unwrap();
    data_res.last_insert_id
}
