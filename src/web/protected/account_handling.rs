use axum::{Router, routing::post};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route(
        "/change_password",
        post(self::post::change_password_protected),
    )
}

mod post {
    use axum::{Json, extract::State, response::IntoResponse};
    use hyper::StatusCode;

    use crate::web::{
        api::Api,
        generic::change_password::{PasswordChange, change_password},
        user::AuthSession,
    };

    pub async fn change_password_protected(
        auth_session: AuthSession,
        State(data): State<Api>,
        Json(new_password): Json<PasswordChange>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let user = auth_session.user.expect("No user in protected space");
        match change_password(db, user.inner.username, new_password.password).await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
