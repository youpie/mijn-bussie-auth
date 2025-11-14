use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    instance_handling::admin::db::get::{get_all_instances, get_all_users},
    web::api::Api,
};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/instances", get(get_all_instances))
        .route("/users", get(get_all_users))
        .route("/import_user", post(self::post::import_user))
}

mod get {
    use axum::{extract::State, response::IntoResponse};
    use entity::{user_account, user_data};
    use reqwest::StatusCode;
    use sea_orm::EntityTrait;

    use crate::web::api::Api;

    pub async fn get_all_instances(State(data): State<Api>) -> impl IntoResponse {
        if let Ok(instances) = user_data::Entity::find().all(&data.db).await {
            let instances: Vec<String> = instances.iter().map(|x| x.user_name.to_owned()).collect();
            (
                StatusCode::OK,
                serde_json::to_string_pretty(&instances).unwrap(),
            )
                .into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }

    pub async fn get_all_users(State(data): State<Api>) -> impl IntoResponse {
        if let Ok(instances) = user_account::Entity::find().all(&data.db).await {
            let instances: Vec<String> = instances.iter().map(|x| x.username.to_owned()).collect();
            (
                StatusCode::OK,
                serde_json::to_string_pretty(&instances).unwrap(),
            )
                .into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

mod post {
    use std::path::PathBuf;

    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;
    use serde::Deserialize;

    use crate::{file_user::transfer::tranfer_user_from_path, web::api::Api};

    #[derive(Deserialize)]
    pub struct PathQuery {
        path: PathBuf,
    }

    pub async fn import_user(
        State(data): State<Api>,
        Query(path): Query<PathQuery>,
    ) -> impl IntoResponse {
        match tranfer_user_from_path(&data.db, &path.path).await {
            Ok(id) => (
                StatusCode::OK,
                format!(
                    "Inserted user {} with ID {id}",
                    path.path.file_stem().unwrap_or_default().to_string_lossy()
                ),
            ),
            Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        }
        .into_response()
    }
}
