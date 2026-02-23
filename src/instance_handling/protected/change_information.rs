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

    use crate::{
        Client, instance_handling::generic::change_information::InstanceInformation, web::{api::Api, user::AuthSession}
    };

    pub async fn change_information_protected(
        auth_session: AuthSession,
        State(data): State<Api>,
        Json(information): Json<InstanceInformation>,
    ) -> impl IntoResponse {
        let user = auth_session.user.expect("No user in protected space");
        let data = data.db;
        let information = information.censor();
        information.change_information_protected(data, user).await
    }
}
