use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    instance_handling::admin::passthrough::{
        get::{get_exit_code, get_logbook},
        post::start_user,
    },
    web::api::Api,
};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/logbook", get(get_logbook))
        .route("/exitcode", get(get_exit_code))
        .route("/start", post(start_user))
}

mod get {
    use axum::{extract::Query, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{admin::AdminQuery, instance_api},
        web::user::AuthSession,
    };

    pub async fn get_logbook(
        Query(user): Query<AdminQuery>,
        auth_session: AuthSession,
    ) -> impl IntoResponse {
        match instance_api::Instance::get_logbook(&user.user_name, &auth_session).await {
            Ok(logbook) => (StatusCode::OK, logbook),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }

    pub async fn get_exit_code(Query(user): Query<AdminQuery>) -> impl IntoResponse {
        match instance_api::Instance::get_exit_code(&user.user_name).await {
            Ok(logbook) => (StatusCode::OK, logbook),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }
}

mod post {
    use axum::{extract::Query, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::instance_handling::{admin::AdminQuery, instance_api};

    pub async fn start_user(Query(user): Query<AdminQuery>) -> impl IntoResponse {
        match instance_api::Instance::start_user(&user.user_name).await {
            Ok(started) => (StatusCode::OK, started.to_string()),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }
}
