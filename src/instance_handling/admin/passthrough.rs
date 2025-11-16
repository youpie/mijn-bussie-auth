use axum::{
    Router,
    routing::{get, post},
};
use serde::Deserialize;
use strum::AsRefStr;

use crate::{
    instance_handling::admin::passthrough::{
        get::instance_get,
        post::{instance_post, refresh_instance},
    },
    web::api::Api,
};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/{request}", get(instance_get))
        .route("/{request}", post(instance_post))
        .route("/refresh", post(refresh_instance))
        .route("/kuma/{request}/{user}", post(self::post::handle_kuma))
        .route("/kuma/{request}", post(self::post::handle_kuma))
}

#[derive(Debug, Deserialize, AsRefStr)]
pub enum InstanceGetRequests {
    Logbook,
    IsActive,
    ExitCode,
}

#[derive(Debug, Deserialize, AsRefStr)]
pub enum InstancePostRequests {
    Start,
}

mod get {
    use axum::{
        extract::{Path, Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            admin::{AdminQuery, passthrough::InstanceGetRequests},
            instance_api,
        },
        web::api::Api,
    };

    pub async fn instance_get(
        State(data): State<Api>,
        Path(request_type): Path<InstanceGetRequests>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::get_request(&instance_name.unwrap_or_default(), request_type)
            .await
        {
            Ok(respone) => respone,
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }
}

mod post {
    use axum::{
        extract::{Path, Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            admin::{AdminQuery, passthrough::InstancePostRequests},
            instance_api::{self, Instance, KumaRequest},
        },
        web::api::Api,
    };

    pub async fn instance_post(
        State(data): State<Api>,
        Path(request_type): Path<InstancePostRequests>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let instance_name = user.get_instance_name(&data.db).await;
        match instance_api::Instance::post_request(&instance_name.unwrap_or_default(), request_type)
            .await
        {
            Ok(response) => response,
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }

    pub async fn refresh_instance(
        State(data): State<Api>,
        Query(user): Query<Option<AdminQuery>>,
    ) -> impl IntoResponse {
        let instance_name = if let Some(query) = user {
            query.get_instance_name(&data.db).await
        } else {
            None
        };
        match instance_api::Instance::refresh_user(instance_name.as_deref()).await {
            Ok(started) => (StatusCode::OK, started.to_string()),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }

    pub async fn handle_kuma(
        Path((request, user)): Path<(KumaRequest, Option<String>)>,
    ) -> impl IntoResponse {
        match Instance::kuma_request(user.as_deref(), request).await {
            Ok(code) => code.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
