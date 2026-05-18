use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/change_password", post(change_password_protected))
        .route("/role", get(role))
}

pub async fn role(auth_session: AuthSession) -> impl IntoResponse {
    (
        StatusCode::OK,
        auth_session
            .user
            .expect("No user in protected space")
            .inner
            .role,
    )
        .into_response()
}

pub async fn change_password_protected(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Json(new_password): Json<PasswordChange>,
) -> impl IntoResponse {
    let db = &data.db;
    let user = auth_session.user.expect("No user in protected space");
    match change_password(db, user.inner.username, new_password.password).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
