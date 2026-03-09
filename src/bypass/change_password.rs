use axum::{Router, routing::post};
use serde::Deserialize;

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route("/change_password", post(self::post::change_password))
}

#[derive(Debug, Deserialize)]
struct PasswordChange {
    calendar_link: String,
    personeelsnummer: String,
    password: String,
}

mod post {
    use axum::{Json, extract::State, response::IntoResponse};
    use entity::user_data;
    use hyper::StatusCode;
    use sea_orm::{ActiveValue::Set, EntityTrait, IntoActiveModel};
    use serde_json::Value;

    use crate::{
        decrypt_value, encrypt_value,
        error::ResultLog,
        instance_handling::instance_api::{Instance, InstanceGetRequests},
        web::api::Api,
    };

    pub async fn change_password(
        State(data): State<Api>,
        Json(new_password): Json<super::PasswordChange>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let normalised_personeelsnummer = new_password
            .personeelsnummer
            .parse::<i32>()
            .unwrap_or_default()
            .to_string();
        let instances = user_data::Entity::find()
            .all(db)
            .await
            .warn_owned("Finding user in bypass password change")
            .unwrap_or_default()
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
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let instance = instances.first().unwrap().clone();

        let calendar_json: Value = serde_json::from_str(
            &Instance::get_request(&instance.user_name, InstanceGetRequests::Calendar)
                .await
                .warn_owned("Getting calendar URL for password change")
                .unwrap_or_default()
                .1,
        )
        .unwrap_or_default();

        let calendar_link = calendar_json["GenResponse"].as_str().unwrap_or_default();

        if calendar_link != new_password.calendar_link {
            println!(
                "User {} tried to change password, but supplied link was incorrect",
                new_password.calendar_link
            );
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let mut active_instance = instance.clone().into_active_model();

        if let Ok(password) = encrypt_value(&new_password.password) {
            active_instance.password = Set(password);
            user_data::Entity::update(active_instance)
                .exec(db)
                .await
                .warn("Updating password");
        } else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        Instance::refresh_user(Some(&instance.user_name))
            .await
            .warn("Refreshing bypassed password change");

        println!("Changed password for user {normalised_personeelsnummer}");

        StatusCode::OK.into_response()
    }
}
