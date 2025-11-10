use std::path::PathBuf;

use dotenvy::var;
use entity::{user_data, user_properties};
use sea_orm::ActiveValue::{NotSet, Set};

use crate::encode_password;

pub fn load_user(path: PathBuf) -> (user_properties::ActiveModel, user_data::ActiveModel) {
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
        password: Set(encode_password(password)),
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
