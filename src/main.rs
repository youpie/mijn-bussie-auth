use std::path::PathBuf;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use clap::{Parser, arg};
use dotenvy::{dotenv_override, var};
use entity::{user_data, user_properties};
use sea_orm::ActiveValue::NotSet;
use sea_orm::Database;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

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
        let id = add_to_db(&db, data).await;
        println!("added user with ID of {id}");
    }
}

fn encode_password(password: String, secret: String) -> String {
    let secret = secret.as_bytes();
    BASE64_STANDARD_NO_PAD
        .encode(simplestcrypt::encrypt_and_serialize(secret, password.as_bytes()).unwrap())
}

async fn add_to_db(
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

fn load_user(
    path: PathBuf,
    secret: String,
) -> (user_properties::ActiveModel, user_data::ActiveModel) {
    let mut env_path = path.clone();
    env_path.push(".env");
    dotenvy::from_filename_override(env_path).unwrap();
    let username = var("USERNAME").unwrap();
    let password = var("PASSWORD").unwrap();
    let filename = var("RANDOM_FILENAME").ok();
    let cycle_time = var("CYCLE_TIME")
        .unwrap_or(
            (var("KUMA_HEARTBEAT_INTERVAL")
                .unwrap()
                .parse::<i32>()
                .unwrap()
                - 400)
                .to_string(),
        )
        .parse::<i32>()
        .unwrap();
    let email_to = var("MAIL_TO").unwrap();
    let new_shift = str_to_bool(var("SEND_EMAIL_NEW_SHIFT").unwrap());
    let updated_shift = str_to_bool(var("SEND_MAIL_UPDATED_SHIFT").unwrap());
    let removed_shift = updated_shift;
    let failed_signin = str_to_bool(var("SEND_MAIL_SIGNIN_FAILED").unwrap());
    let welcome_mail = str_to_bool(var("SEND_WELCOME_MAIL").unwrap());
    let error_mail = str_to_bool(var("SEND_ERROR_MAIL").unwrap());
    let split_night_shift = str_to_bool(var("BREAK_UP_NIGHT_SHIFT").unwrap());
    let stop_night_shift = str_to_bool(var("STOP_SHIFT_AT_MIDNIGHT").unwrap_or("false".to_owned()));
    let mut execution_min_path = path;
    execution_min_path.push("kuma");
    execution_min_path.push("starting_minute");
    let execution_min = std::fs::read_to_string(execution_min_path)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let user_properties = user_properties::ActiveModel {
        execution_interval_minutes: Set(cycle_time),
        execution_minute: Set(execution_min),
        send_error_mail: Set(error_mail),
        send_failed_signin_mail: Set(failed_signin),
        send_mail_new_shift: Set(new_shift),
        send_mail_removed_shift: Set(removed_shift),
        send_mail_updated_shift: Set(updated_shift),
        send_welcome_mail: Set(welcome_mail),
        stop_midnight_shift: Set(stop_night_shift),
        split_night_shift: Set(split_night_shift),
        ..Default::default()
    };
    let user_data = user_data::ActiveModel {
        user_name: Set(username.clone()),
        personeelsnummer: Set(username.clone()),
        password: Set(encode_password(password, secret)),
        email: Set(email_to),
        file_name: Set(filename.unwrap_or(username)),
        user_properties: NotSet,
        ..Default::default()
    };
    (user_properties, user_data)
}

fn str_to_bool(input: String) -> bool {
    match input.as_str() {
        "true" => true,
        _ => false,
    }
}
