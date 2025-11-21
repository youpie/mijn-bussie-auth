use axum::Router;
use axum::routing::get;
use axum::routing::post;
use bcrypt::DEFAULT_COST;
use entity::user_account;
use sea_orm::ActiveValue::Set;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::QueryFilter;

use crate::GenResult;
use crate::web::api::Api;
use crate::web::user::{AuthSession, Credentials};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/logout", get(self::get::logout))
}

mod post {

    use super::*;

    use axum::{Json, http::StatusCode, response::IntoResponse};

    pub async fn login(
        mut auth_session: AuthSession,
        Json(creds): Json<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        auth_session.user.unwrap();
        StatusCode::OK.into_response()
    }
}

mod get {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

use sea_orm::ColumnTrait;

pub async fn change_password(db: &DatabaseConnection, username: String, new_password: String) -> GenResult<()> {
    let user_model = user_account::Entity::find().filter(user_account::Column::Username.eq(username)).one(db).await?;
    if let Some(user_model) = user_model {
        let mut active_model = user_model.into_active_model();
        active_model.password_hash = Set(tokio::task::spawn_blocking(|| bcrypt::hash(new_password, DEFAULT_COST)).await??);
        user_account::Entity::update(active_model).validate()?.exec(db).await?;
        Ok(())
    } else {
        Err("User not found".into())
    }
    
}
