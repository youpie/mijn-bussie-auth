use axum::{
    Router,
    routing::{get, post},
};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new()
        .route("/{request}", get(self::get::get_instance))
        .route("/{request}", post(self::post::post_instance))
}

mod get {
    use axum::{extract::Path, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::{
        instance_handling::instance_api::{self, InstanceGetRequests},
        web::user::{AuthSession, GetUser},
    };

    pub async fn get_instance(
        auth_session: AuthSession,
        Path(request_type): Path<InstanceGetRequests>,
    ) -> impl IntoResponse {
        let user = match auth_session.get_user() {
            Ok(user) => user,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        match instance_api::Instance::get_request(
            &user.inner.backend_user.unwrap_or_default(),
            request_type,
        )
        .await
        {
            Ok(link) if link.0 == StatusCode::OK => link.into_response(),
            Ok(link) => link.0.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

mod post {
    use axum::{extract::Path, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::{
        instance_handling::instance_api::{self, InstancePostRequests},
        web::user::{AuthSession, GetUser},
    };

    pub async fn post_instance(
        auth_session: AuthSession,
        Path(request_type): Path<InstancePostRequests>,
    ) -> impl IntoResponse {
        let user = match auth_session.get_user() {
            Ok(user) => user,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        match instance_api::Instance::post_request(
            &user.inner.backend_user.unwrap_or_default(),
            request_type,
        )
        .await
        {
            Ok(link) if link.0 == StatusCode::OK => link.into_response(),
            Ok(link) => link.0.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
