use axum::{Router, routing::post};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route("/change_password", post(self::post::change_password_admin))
}

mod post {
    use axum::{
        Json,
        extract::{Query, State},
        response::IntoResponse,
    };
    use hyper::StatusCode;

    use crate::{
        instance_handling::{admin::AdminQuery, generic::change_password::post::PasswordChange},
        web::{api::Api, auth::change_password},
    };

    pub async fn change_password_admin(
        State(data): State<Api>,
        Query(query): Query<AdminQuery>,
        Json(new_password): Json<PasswordChange>,
    ) -> impl IntoResponse {
        if new_password.password.is_empty() {
            return StatusCode::NOT_ACCEPTABLE.into_response();
        }
        let db = &data.db;
        match change_password(
            db,
            query.account_name.unwrap_or_default(),
            new_password.password,
        )
        .await
        {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
