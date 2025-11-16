use axum::{
    Router,
    routing::{get, post},
};

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

mod get {
    use axum::{
        extract::{Path, Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            admin::AdminQuery,
            instance_api::{self, InstanceGetRequests},
        },
        web::api::Api,
    };

    pub async fn instance_get(
        State(data): State<Api>,
        Path(request_type): Path<InstanceGetRequests>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let instance_name =
            match AdminQuery::map_instance_query_result(user.get_instance_name(db).await) {
                Ok(name) => name,
                Err(names) => return names.into_response(),
            };
        match instance_api::Instance::get_request(&instance_name, request_type).await {
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
            admin::AdminQuery,
            instance_api::{self, Instance, InstancePostRequests, KumaRequest},
        },
        web::api::Api,
    };

    pub async fn instance_post(
        State(data): State<Api>,
        Path(request_type): Path<InstancePostRequests>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let instance_name =
            match AdminQuery::map_instance_query_result(user.get_instance_name(db).await) {
                Ok(name) => name,
                Err(names) => return names.into_response(),
            };
        match instance_api::Instance::post_request(&instance_name, request_type).await {
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
            match AdminQuery::map_instance_query_result(query.get_instance_name(&data.db).await) {
                Ok(name) => Some(name),
                Err(err) => return err.into_response(),
            }
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
