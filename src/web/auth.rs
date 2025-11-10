use axum::Router;
use axum::routing::get;
use axum::routing::post;

use crate::web::user::{AuthSession, Credentials};

pub fn router() -> Router<()> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/logout", get(self::get::logout))
}

mod post {

    use super::*;

    use axum::{Json, http::StatusCode, response::IntoResponse};

    pub async fn login(
        mut auth_session: AuthSession,
        Json(creds): Json<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        StatusCode::OK.into_response()
    }
}

mod get {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
