use std::time::Duration;

use axum::{
    BoxError, Router,
    error_handling::HandleErrorLayer,
    routing::{get, put},
};
use hyper::StatusCode;
use tower::{ServiceBuilder, buffer::BufferLayer, limit::RateLimitLayer};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    let layer = RateLimitLayer::new(1, Duration::from_secs(1));
    Router::new()
        .route("/instance/{user_name}", get(self::get::get_instance_state))
        .route("/instance", put(self::put::create_instance))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::TOO_MANY_REQUESTS,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(layer),
        )
}

mod put {
    use axum::{Json, debug_handler, extract::State, response::IntoResponse};
    use hyper::StatusCode;
    use serde_json::Value;

    use crate::{
        Client,
        error::ResultLog,
        instance_handling::{
            entity::{InstanceMatchReturn, MijnBussieInstance},
            instance_api::*,
        },
        web::api::Api,
    };

    #[debug_handler]
    pub async fn create_instance(
        State(data): State<Api>,
        Json(new_user): Json<MijnBussieInstance>,
    ) -> impl IntoResponse {
        let db = data.db;
        let mut cencored_user = new_user.censor();
        cencored_user.online_created = true;
        let inserted = cencored_user
            .create_and_insert_instance(&db, false)
            .await
            .warn_owned("inserting user");
        if let Ok(inserted_instance) = inserted {
            match inserted_instance {
                InstanceMatchReturn::NewUser(instance) => {
                    Instance::refresh_user(Some(&instance.user_name))
                        .await
                        .unwrap();
                    Instance::post_request(&instance.user_name, InstancePostRequests::Start)
                        .await
                        .unwrap();
                    (StatusCode::OK, instance.user_name).into_response()
                }
                InstanceMatchReturn::Exact(instance) => {
                    let calendar_link_response =
                        Instance::get_request(&instance.user_name, InstanceGetRequests::Calendar)
                            .await
                            .unwrap_or_default();
                    let calendar_link = serde_json::from_str::<Value>(&calendar_link_response.1)
                        .map_or(Value::Null, |value| value["GenResponse"].clone());
                    if calendar_link_response.0 != StatusCode::OK && calendar_link != Value::Null {
                        println!("Non OK exit code {calendar_link:?}");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                    (
                        StatusCode::FOUND,
                        calendar_link.as_str().unwrap_or_default().to_owned(),
                    )
                        .into_response()
                }
                InstanceMatchReturn::Partial => (StatusCode::NOT_ACCEPTABLE).into_response(),
            }
        } else {
            println!("{inserted:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

mod get {
    use axum::{extract::Path, response::IntoResponse};
    use hyper::StatusCode;
    use serde_json::{Value, json};

    use crate::{error::ResultLog, instance_handling::instance_api::*};

    pub async fn get_instance_state(Path(user_name): Path<String>) -> impl IntoResponse {
        let active = Instance::get_request(&user_name, InstanceGetRequests::IsActive)
            .await
            .warn_owned("Active Request");
        if let Ok(instance_active_response) = active.as_ref()
            && instance_active_response.0 == StatusCode::OK
            && let Ok(instance_active) = serde_json::from_str::<Value>(&instance_active_response.1)
        {
            if instance_active == json!({"Active":"Dirty"}) {
                println!("{instance_active:?}");
                Instance::post_request(&user_name, InstancePostRequests::Delete)
                    .await
                    .warn("Removing temp user");
                StatusCode::NOT_ACCEPTABLE
            } else if instance_active == json!({"Active":"Active"}) {
                StatusCode::TOO_EARLY
            } else if instance_active == json!({"Active":"SignedIn"}) {
                StatusCode::ACCEPTED
            } else {
                StatusCode::OK
            }
        } else {
            println!("Active Request failed {active:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        .into_response()
    }
}
