use axum::{Router, routing::get};

use crate::{
    instance_handling::protected::passthrough::get::{
        get_calendar_link_protected, get_is_active_protected, get_logbook_protected,
    },
    web::api::Api,
};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/calendar", get(get_calendar_link_protected))
        .route("/logbook", get(get_logbook_protected))
        .route("/isactive", get(get_is_active_protected))
}

mod get {
    use axum::response::IntoResponse;
    use reqwest::StatusCode;

    use crate::{
        instance_handling::instance_api,
        web::user::{AuthSession, GetUser},
    };

    pub async fn get_calendar_link_protected(auth_session: AuthSession) -> impl IntoResponse {
        let user = match auth_session.get_user() {
            Ok(user) => user,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        match instance_api::Instance::get_calendar_link(&user.inner.username).await {
            Ok(link) if link.0 == StatusCode::OK => link.into_response(),
            Ok(link) => link.0.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn get_logbook_protected(auth_session: AuthSession) -> impl IntoResponse {
        let user = match auth_session.get_user() {
            Ok(user) => user,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        match instance_api::Instance::get_logbook(&user.inner.username).await {
            Ok(link) if link.0 == StatusCode::OK => link.into_response(),
            Ok(link) => link.0.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn get_is_active_protected(auth_session: AuthSession) -> impl IntoResponse {
        let user = match auth_session.get_user() {
            Ok(user) => user,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        match instance_api::Instance::get_is_active(&user.inner.username).await {
            Ok(link) if link.0 == StatusCode::OK => link.into_response(),
            Ok(link) => link.0.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
