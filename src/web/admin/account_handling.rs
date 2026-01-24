use axum::{Router, routing::post};

use crate::web::api::Api;

pub fn router() -> Router<Api> {
    Router::new().route("/change_password", post(self::post::change_password_admin))
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
    use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
    use serde::Deserialize;

    use crate::{
        GenResult, OptionResult,
        instance_handling::admin::AdminQuery,
        web::{
            api::Api,
            generic::change_password::{PasswordChange, change_password},
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
    struct NewRole {
        pub role: String,
    }

    pub async fn change_role(
        auth_session: AuthSession,
        State(data): State<Api>,
        Query(query): Query<AdminQuery>,
        Json(new_role): Json<NewRole>,
    ) -> impl IntoResponse {
        let selected_user = query.account_name.unwrap_or_default();
        if auth_session
            .user
            .is_some_and(|auth_user| auth_user.inner.username == selected_user)
        {
            return (StatusCode::NOT_ACCEPTABLE, "Can't change own role!").into_response();
        }
        match Role::from_str(new_role) {
            Ok(new_role) => {}
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

    fn change_role_error(
        db: &DatabaseConnection,
        user_account: String,
        new_role: Role,
    ) -> GenResult<()> {
        let user_account = user_account::Entity::find()
            .filter(user_account::Column::Username.contains(selected_user))
            .one(db)
            .await?
            .result_reason("User Not found");
    }
}
