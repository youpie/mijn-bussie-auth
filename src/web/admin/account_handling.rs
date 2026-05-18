use sea_orm::{ActiveValue::Set, IntoActiveModel, PaginatorTrait};
use serde::Deserialize;

use crate::web::user::Role;

use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/change_password", post(change_password_admin))
        .route("/role", post(change_role))
        .route("/role", get(role))
        .route("/delete_account", post(delete_account))
}

pub async fn role(
    State(data): State<AppState>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    match query
        .get_user_account(&data.db, true)
        .await
        .and_then(|account| Some(account.inner.role))
    {
        Some(role) => (StatusCode::OK, role),
        None => (StatusCode::NOT_FOUND, "Account niet gevonden".to_owned()),
    }
    .into_response()
}

pub async fn change_password_admin(
    State(data): State<AppState>,
    Query(query): Query<AdminQuery>,
    Json(new_password): Json<generic::account_handling::PasswordChange>,
) -> GenResult<StatusCode> {
    if let Some(password) = new_password.password {
        let db = &data.db;
        change_password(db, query.account_name.unwrap_or_default(), password).await?;
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::BAD_REQUEST)
    }
}

#[derive(Debug, Deserialize)]
pub struct NewRole {
    pub role: Role,
}

pub async fn change_role(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Query(query): Query<AdminQuery>,
    Json(new_role): Json<NewRole>,
) -> GenResult<impl IntoResponse> {
    let selected_user = query.account_name.as_deref().unwrap_or_default();
    if auth_session
        .user
        .is_some_and(|auth_user| &auth_user.inner.username == selected_user)
    {
        return Ok((StatusCode::NOT_ACCEPTABLE, "Can't change own role!").into_response());
    }
    let db = &data.db;
    let user_account = find_user_account(db, &query).await?;
    let mut active_account = user_account.into_active_model();
    active_account.role = Set(new_role.role.as_ref().to_owned());
    user_account::Entity::update(active_account)
        .validate()?
        .exec(db)
        .await?;
    Ok(().into_response())
}

pub async fn delete_account(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Query(query): Query<AdminQuery>,
) -> GenResult<impl IntoResponse> {
    let db = &data.db;
    let authenticated_user = auth_session.user.expect("No user in Admin space");

    // Only allow the admin to delete their account if they are the only user
    let current_accounts = user_account::Entity::find().count(db).await?;
    let user_account = query.get_user_account(db, true).await.result()?;

    if current_accounts != 1 && user_account.inner.username == authenticated_user.inner.username {
        return Ok((StatusCode::NOT_ACCEPTABLE, "Can't delete own account").into_response());
    }
    user_account::Entity::delete(user_account.inner.into_active_model())
        .exec(db)
        .await?;
    Ok(().into_response())
}
