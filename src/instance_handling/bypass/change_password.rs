use sea_orm::{ActiveValue::Set, EntityTrait, IntoActiveModel};
use serde::Deserialize;
use serde_json::Value;

use crate::crypt::{decrypt_value, encrypt_value};

use super::*;

pub fn router() -> Router<AppState> {
    Router::new().route("/change_password", post(change_password))
}

#[derive(Debug, Deserialize)]
struct BypassPasswordChange {
    calendar_link: String,
    personeelsnummer: String,
    password: String,
}

async fn change_password(
    State(data): State<AppState>,
    Json(new_password): Json<BypassPasswordChange>,
) -> GenResult<StatusCode> {
    let db = &data.db;
    let normalised_personeelsnummer = new_password
        .personeelsnummer
        .parse::<i32>()
        .d()?
        .to_string();
    let instances = user_data::Entity::find()
        .all(db)
        .await
        .warn_owned("Finding user in bypass password change")?
        .into_iter()
        .filter(|value| value.user_name == normalised_personeelsnummer)
        .collect::<Vec<_>>();
    if instances.len() != 1 {
        println!(
            "Found {} matches for user {}, so can't change password: {:?}",
            instances.len(),
            new_password.personeelsnummer,
            instances
                .into_iter()
                .map(|x| decrypt_value(&x.personeelsnummer, false).unwrap_or_default())
                .collect::<Vec<String>>()
        );
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let instance = instances.first().unwrap().clone();

    let calendar_json: Value = serde_json::from_str(
        &get_request(
            &data.client,
            &instance.user_name,
            InstanceGetRequests::Calendar,
        )
        .await?,
    )?;

    let calendar_link = calendar_json["GenResponse"].as_str().unwrap_or_default();

    if calendar_link != new_password.calendar_link {
        println!(
            "User {} tried to change password, but supplied link was incorrect",
            new_password.calendar_link
        );
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut active_instance = instance.clone().into_active_model();

    let password = encrypt_value(&new_password.password)?;
    active_instance.password = Set(password);
    user_data::Entity::update(active_instance)
        .exec(db)
        .await
        .warn("Updating password");

    refresh_user(&data.client, Some(&instance.user_name))
        .await
        .warn("Refreshing bypassed password change");

    println!("Changed password for user {normalised_personeelsnummer}");

    Ok(StatusCode::OK)
}
