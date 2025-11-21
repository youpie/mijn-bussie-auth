use axum::{Router, routing::post};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route(
        "/change_instance_password",
        post(self::post::change_password_protected),
    )
}

mod post {
    use axum::{Json, extract::State, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::{
        instance_handling::generic::change_password::post::{PasswordChange, change_password},
        web::{api::Api, user::AuthSession},
    };

    pub async fn change_password_protected(
        auth_session: AuthSession,
        State(data): State<Api>,
        Json(password): Json<PasswordChange>,
    ) -> impl IntoResponse {
        let user = auth_session.user.expect("No user in protected space");
        let response = if let Ok(Some(instance_data)) = user.get_instance_data(&data.db).await {
            match change_password(&data.db, &instance_data, &password).await {
                Ok(response) => response.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        } else {
            StatusCode::NO_CONTENT.into_response()
        };
        response
    }
}
