use axum::Router;
use axum::routing::get;

pub fn router() -> Router<()> {
    Router::new().route("/", get(self::get::protected))
}

mod get {
    use axum::{http::StatusCode, response::IntoResponse};

    use crate::web::user::AuthSession;

    pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => StatusCode::OK.into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
