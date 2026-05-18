use axum::{Json, Router, response::IntoResponse, routing::post};

use crate::web::{api::AppState, user::Credentials};

pub fn router() -> Router<AppState> {
    Router::new().route("/signup", post(self::post::create_user))
}

mod post {
    use axum::{extract::State, http::StatusCode};

    use crate::web::user::UserAccount;

    use super::*;

    pub async fn create_user(
        State(data): State<AppState>,
        Json(user): Json<Credentials>,
    ) -> impl IntoResponse {
        match UserAccount::add_user(&data.db, user).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(error) => (StatusCode::NOT_ACCEPTABLE, error.to_string()).into_response(),
        }
    }
}
