use axum::Router;

use crate::web::api::Api;
use axum::routing::get;

pub fn router() -> Router<Api> {
    Router::new()
        .route("/names", get(self::get::get_name_list))
        .route("/emails", get(self::get::get_email_list))
}

mod get {
    use axum::{Json, extract::State, response::IntoResponse};
    use hyper::StatusCode;
    use sea_orm::DatabaseConnection;

    use crate::{GenResult, instance_handling::entity::MijnBussieUser, web::api::Api};

    pub async fn get_email_list(State(data): State<Api>) -> impl IntoResponse {
        let db = &data.db;
        match get_email_list_error(db).await {
            Ok(list) => (StatusCode::OK, Json(list)).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }

    async fn get_email_list_error(db: &DatabaseConnection) -> GenResult<Vec<String>> {
        let all_users = MijnBussieUser::get_all_users(db).await?;
        let email_list = all_users
            .iter()
            .filter_map(|user| user.get_email().ok())
            .collect();
        Ok(email_list)
    }

    pub async fn get_name_list(State(data): State<Api>) -> impl IntoResponse {
        let db = &data.db;
        match get_name_list_error(db).await {
            Ok(list) => (StatusCode::OK, Json(list)).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }

    async fn get_name_list_error(db: &DatabaseConnection) -> GenResult<Vec<String>> {
        let all_users = MijnBussieUser::get_all_users(db).await?;
        let name_list = all_users
            .iter()
            .filter_map(|user| user.get_name().ok())
            .collect();
        Ok(name_list)
    }
}
