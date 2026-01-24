use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new()
        .route("/change_password", post(self::post::change_password_admin))
        .route("/role", post(self::post::change_role))
        .route("/role", get(self::get::role))
        .route("/delete_account", delete(self::delete::delete_account))
}

mod get {
    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use hyper::StatusCode;

    use crate::{instance_handling::admin::AdminQuery, web::api::Api};

    pub async fn role(
        State(data): State<Api>,
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
}

mod post {
    use std::str::FromStr;

    use axum::{
        Json,
        extract::{Query, State},
        response::IntoResponse,
    };
    use entity::user_account;
    use hyper::StatusCode;
    use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};
    use serde::Deserialize;

    use crate::{
        GenResult,
        instance_handling::admin::AdminQuery,
        web::{
            api::Api,
            generic::account_management::{PasswordChange, change_password},
            user::{AuthSession, Role},
        },
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

    #[derive(Debug, Deserialize)]
    pub struct NewRole {
        pub role: String,
    }

    pub async fn change_role(
        auth_session: AuthSession,
        State(data): State<Api>,
        Query(query): Query<AdminQuery>,
        Json(new_role): Json<NewRole>,
    ) -> impl IntoResponse {
        let selected_user = query.account_name.as_deref().unwrap_or_default();
        if auth_session
            .user
            .is_some_and(|auth_user| &auth_user.inner.username == selected_user)
        {
            return (StatusCode::NOT_ACCEPTABLE, "Can't change own role!").into_response();
        }
        let db = &data.db;
        let response = if let Ok(new_role) = Role::from_str(&new_role.role) {
            match change_role_error(db, &query, new_role).await {
                Ok(_) => (
                    StatusCode::OK,
                    format!(
                        "Role of user {selected_user} was changed to {}",
                        new_role.as_ref()
                    ),
                ),
                Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            }
        } else {
            (StatusCode::NOT_ACCEPTABLE, "Role not found".to_owned())
        };
        response.into_response()
    }

    async fn change_role_error(
        db: &DatabaseConnection,
        user_account: &AdminQuery,
        new_role: Role,
    ) -> GenResult<()> {
        let user_account = super::super::find_user_account(db, user_account).await?;
        let mut active_account = user_account.into_active_model();
        active_account.role = Set(new_role.as_ref().to_owned());
        user_account::Entity::update(active_account)
            .validate()?
            .exec(db)
            .await?;
        Ok(())
    }
}

mod delete {
    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use entity::user_account;
    use hyper::StatusCode;
    use sea_orm::{EntityTrait, IntoActiveModel, PaginatorTrait};

    use crate::{
        instance_handling::admin::AdminQuery,
        web::{api::Api, user::AuthSession},
    };

    pub async fn delete_account(
        auth_session: AuthSession,
        State(data): State<Api>,
        Query(query): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let authenticated_user = auth_session.user.expect("No user in Admin space");

        // Only allow the admin to delete their account if they are the only user
        let current_accounts = user_account::Entity::find().count(db).await.unwrap_or(0);
        let user_account = query
            .get_user_account(db, true)
            .await
            .unwrap_or(authenticated_user.clone());

        if current_accounts == 1 && user_account.inner.username == authenticated_user.inner.username
        {
            return (StatusCode::NOT_ACCEPTABLE, "Can't delete own account").into_response();
        }

        match user_account::Entity::delete(user_account.inner.into_active_model())
            .exec(db)
            .await
        {
            Ok(_) => (StatusCode::OK, "Removed Account".to_owned()),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
        .into_response()
    }
}
