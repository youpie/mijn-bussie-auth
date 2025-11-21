use std::path::PathBuf;

use dotenvy::EnvLoader;
use entity::{user_data, user_properties};
use sea_orm::ActiveValue::{NotSet, Set};

use crate::{GenResult, encrypt_value};

fn get_name(base_path: &PathBuf) -> Option<String> {
    let mut path = base_path.clone();
    path.push("kuma/name");
    std::fs::read_to_string(path).ok()
}

pub fn load_user(
    path: &PathBuf,
) -> GenResult<(user_properties::ActiveModel, user_data::ActiveModel)> {
    let mut env_path = path.clone();
    env_path.push(".env");

    let env_map = EnvLoader::with_path(env_path).load()?;

    let username = env_map.var("USERNAME")?;
    let password = env_map.var("PASSWORD")?;
    let filename = env_map.var("RANDOM_FILENAME").ok();
    let cycle_time = env_map
        .var("CYCLE_TIME")
        .unwrap_or((env_map.var("KUMA_HEARTBEAT_INTERVAL")?.parse::<i32>()? - 500).to_string())
        .parse::<i32>()?
        / 60
        / 60
        * 60;
    let email_to = env_map.var("MAIL_TO")?;
    let new_shift = str_to_bool(
        env_map
            .var("SEND_EMAIL_NEW_SHIFT")
            .unwrap_or("true".to_owned()),
    );
    let updated_shift = str_to_bool(
        env_map
            .var("SEND_MAIL_UPDATED_SHIFT")
            .unwrap_or("true".to_owned()),
    );
    let removed_shift = updated_shift;
    let failed_signin = str_to_bool(
        env_map
            .var("SEND_MAIL_SIGNIN_FAILED")
            .unwrap_or("true".to_owned()),
    );
    let welcome_mail = str_to_bool(
        env_map
            .var("SEND_WELCOME_MAIL")
            .unwrap_or("true".to_owned()),
    );
    let error_mail = str_to_bool(env_map.var("SEND_ERROR_MAIL").unwrap_or("false".to_owned()));
    let split_night_shift = str_to_bool(
        env_map
            .var("BREAK_UP_NIGHT_SHIFT")
            .unwrap_or("false".to_owned()),
    );
    let stop_night_shift = str_to_bool(
        env_map
            .var("STOP_SHIFT_AT_MIDNIGHT")
            .unwrap_or("false".to_owned()),
    );
    let name = get_name(&path);
    let mut execution_min_path = path.clone();
    execution_min_path.push("kuma");
    execution_min_path.push("starting_minute");
    let execution_min = std::fs::read_to_string(execution_min_path)?.parse::<i32>()?;
    let user_properties = user_properties::ActiveModel {
        user_properties_id: NotSet,
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
        auto_delete_account: Set(true),
    };
    let user_data = user_data::ActiveModel {
        user_data_id: NotSet,
        user_name: Set(username.parse::<u32>().unwrap().to_string()),
        personeelsnummer: Set(encrypt_value(&username)?),
        password: Set(encrypt_value(&password)?),
        email: Set(encrypt_value(&email_to)?),
        file_name: Set(filename.unwrap_or(username)),
        name: Set(name.map(|name| encrypt_value(&name).unwrap_or_default())),
        custom_general_properties: NotSet,
        user_properties: NotSet,
        last_execution_date: NotSet,
        last_succesfull_sign_in_date: NotSet,
        creation_date: Set(chrono::offset::Utc::now().naive_utc()),
    };
    Ok((user_properties, user_data))
}

fn str_to_bool(input: String) -> bool {
    match input.as_str() {
        "true" => true,
        _ => false,
    }
}
