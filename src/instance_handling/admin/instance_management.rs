use crate::web::api::Api;
use axum::Router;
use axum::routing::post;

pub fn router() -> Router<Api> {
    Router::new()
        .route("/add_instance", post(self::post::create_instance_admin))
        .route("/change_password", post(self::post::change_password_admin))
}

mod post {
    use axum::{
        Json,
        extract::{Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            admin::AdminQuery,
            entity::MijnBussieUser,
            generic::change_password::post::{PasswordChange, change_password},
        },
        web::api::Api,
    };

    pub async fn create_instance_admin(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
        Json(instance): Json<MijnBussieUser>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let _user = match user.get_user_account(db).await {
            Some(user) => user.get_instance_data(db).await.ok().flatten(),
            None => {
                return (StatusCode::NOT_FOUND, "User not found").into_response();
            }
        };
        match MijnBussieUser::create_and_insert_models(instance, db, None).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }

    pub async fn change_password_admin(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
        Json(password): Json<PasswordChange>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let user = match user.get_user_instance(db).await {
            Some(user) => user,
            None => {
                return (StatusCode::NOT_FOUND, "User not found").into_response();
            }
        };
        match change_password(db, &user, &password).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
