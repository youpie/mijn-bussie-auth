use std::time::Duration;

use axum::{BoxError, Router, error_handling::HandleErrorLayer};
use hyper::StatusCode;
use tower::{ServiceBuilder, buffer::BufferLayer, limit::RateLimitLayer};

use serde_json::{Value, json};

use super::*;

pub fn router() -> Router<AppState> {
    let layer = RateLimitLayer::new(1, Duration::from_secs(1));
    Router::new()
        .route("/instance/{user_name}", get(get_instance_state))
        .route("/instance", post(create_instance))
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

pub async fn create_instance(
    State(data): State<AppState>,
    Json(new_user): Json<MijnBussieInstance>,
) -> GenResult<(StatusCode, String)> {
    let db = data.db;
    let client = data.client;
    let mut cencored_user = new_user.censor();
    cencored_user.online_created = true;
    let inserted = cencored_user
        .create_and_insert_instance(&db, false)
        .await
        .warn_owned("inserting user")?;
    match inserted {
        InstanceMatchReturn::NewUser(instance) => {
            refresh_user(&client, Some(&instance.user_name)).await?;
            post_request(&client, &instance.user_name, InstancePostRequests::Start).await?;
            Ok((StatusCode::OK, instance.user_name))
        }
        InstanceMatchReturn::Exact(instance) => {
            let calendar_link_response =
                get_request(&client, &instance.user_name, InstanceGetRequests::Calendar).await?;
            let calendar_link = serde_json::from_str::<Value>(&calendar_link_response)
                .map_or(Value::Null, |value| value["GenResponse"].clone());
            Ok((
                StatusCode::FOUND,
                calendar_link.as_str().unwrap_or_default().to_owned(),
            ))
        }
        InstanceMatchReturn::Partial => Ok((StatusCode::NOT_ACCEPTABLE, "".to_string())),
    }
}

pub async fn get_instance_state(
    State(data): State<AppState>,
    Path(user_name): Path<String>,
) -> GenResult<StatusCode> {
    let client = data.client;
    let active = get_request(&client, &user_name, InstanceGetRequests::IsActive).await?;
    let instance_active = serde_json::from_str::<Value>(&active)?;
    if instance_active == json!({"Active":"Dirty"}) {
        println!("{instance_active:?}");
        post_request(&client, &user_name, InstancePostRequests::Delete).await?;
        Ok(StatusCode::NOT_ACCEPTABLE)
    } else if instance_active == json!({"Active":"Active"}) {
        Ok(StatusCode::TOO_EARLY)
    } else if instance_active == json!({"Active":"SignedIn"}) {
        Ok(StatusCode::ACCEPTED)
    } else {
        Ok(StatusCode::OK)
    }
}
