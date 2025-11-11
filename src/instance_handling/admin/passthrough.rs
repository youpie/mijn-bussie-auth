use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    instance_handling::admin::passthrough::{
        get::{get_exit_code, get_is_active, get_logbook},
        post::{refresh_instance, start_instance},
    },
    web::api::Api,
};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/logbook", get(get_logbook))
        .route("/exitcode", get(get_exit_code))
        .route("/isactive", get(get_is_active))
        .route("/start", post(start_instance))
        .route("/refresh", post(refresh_instance))
}

mod get {
    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{admin::AdminQuery, instance_api},
        web::api::Api,
    };

    pub async fn get_logbook(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::get_logbook(&instance_name.unwrap_or_default()).await {
            Ok(respone) => respone,
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.source().unwrap().to_string(),
            ),
        }
        .into_response()
    }

    pub async fn get_exit_code(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::get_exit_code(&instance_name.unwrap_or_default()).await {
            Ok(response) => response,
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }

    pub async fn get_is_active(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::get_is_active(&instance_name.unwrap_or_default()).await {
            Ok(response) => response,
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }
}

mod post {
    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{admin::AdminQuery, instance_api},
        web::api::Api,
    };

    pub async fn start_instance(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::start_user(&instance_name.unwrap_or_default()).await {
            Ok(started) => (StatusCode::OK, started.to_string()),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }

    pub async fn refresh_instance(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::refresh_user(&instance_name.unwrap_or_default()).await {
            Ok(started) => (StatusCode::OK, started.to_string()),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }
}
