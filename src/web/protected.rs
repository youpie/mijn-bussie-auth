use axum::Router;
use axum::routing::get;

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route("/me", get(self::get::protected))
}

mod get {
    use axum::{http::StatusCode, response::IntoResponse};

    use crate::web::user::AuthSession;

    pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => (StatusCode::OK, user.inner.username).into_response(),

            None => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}
