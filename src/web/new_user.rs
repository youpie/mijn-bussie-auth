use axum::{Json, Router, response::IntoResponse, routing::post};

use crate::web::{api::Api, user::Credentials};

pub fn router() -> Router<Api> {
    Router::new().route("/signup", post(self::post::create_user))
}

mod post {
    use axum::{extract::State, http::StatusCode};

    use crate::web::user::UserAccount;

    use super::*;

    pub async fn create_user(
        State(data): State<Api>,
        Json(user): Json<Credentials>,
    ) -> impl IntoResponse {
        match UserAccount::add_user(&data.db, user).await {
            Ok(_) => StatusCode::OK,
            Err(error) => {
                println!("{}", error.to_string());
                StatusCode::NOT_ACCEPTABLE
            }
        }
        .into_response()
    }
}
