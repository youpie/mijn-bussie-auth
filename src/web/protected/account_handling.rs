use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/change_password", post(change_password_protected))
        .route("/role", get(role))
}

pub async fn role(auth_session: AuthSession) -> GenResult<String> {
    Ok(auth_session.get_user()?.inner.role)
}

pub async fn change_password_protected(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Json(new_password): Json<PasswordChange>,
) -> GenResult<StatusCode> {
    if let Some(password) = new_password.password {
        let db = &data.db;
        let user = auth_session.get_user()?;
        change_password(db, user.inner.username, password).await?;
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::BAD_REQUEST)
    }
}
