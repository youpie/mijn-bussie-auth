use axum::{Router, routing::get};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route("/me", get(self::get::me))
}

mod get {
    use axum::response::IntoResponse;
    use hyper::StatusCode;

    use crate::web::user::AuthSession;

    pub async fn me(auth_session: AuthSession) -> impl IntoResponse {
        if let Some(user) = auth_session.user {
            (StatusCode::OK, user.inner.username).into_response()
        } else {
            StatusCode::UNAUTHORIZED.into_response()
        }
    }
}