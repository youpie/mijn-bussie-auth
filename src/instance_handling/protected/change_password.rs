use axum::{Router, routing::post};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route(
        "/change_instance_information",
        post(self::post::change_information_protected),
    )
}

mod post {
    use axum::{Json, extract::State, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            entity::MijnBussieInstance, generic::change_information::InstanceInformation,
        },
        web::{api::Api, user::AuthSession},
    };

    pub async fn change_information_protected(
        auth_session: AuthSession,
        State(data): State<Api>,
        Json(information): Json<MijnBussieInstance>,
    ) -> impl IntoResponse {
        let user = auth_session.user.expect("No user in protected space");

        // The user should only be able to change the email and password of the instance
        let information = information.censor();

        let response = if let Ok(Some(instance_data)) = user.get_instance_data(&data.db).await {
            match information
                .change_information(&data.db, &instance_data)
                .await
            {
                Ok(response) => response.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        } else {
            StatusCode::NO_CONTENT.into_response()
        };
        response
    }
}
