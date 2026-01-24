use axum::Router;

use crate::web::api::Api;
use axum::routing::get;

pub fn router() -> Router<Api> {
    Router::new()
        .route("/names", get(self::get::get_name_list))
        .route("/emails", get(self::get::get_email_list))
        .route("/accounts", get(self::get::get_account_list))
}

mod get {
    use axum::{
        Json,
        extract::{Query, State},
        response::IntoResponse,
    };
    use entity::user_account;
    use hyper::StatusCode;
    use sea_orm::{DatabaseConnection, EntityTrait};

    use crate::{
        instance_handling::{admin::AdminQuery, entity::MijnBussieInstance},
        web::api::Api,
    };

    pub async fn get_email_list(
        Query(user): Query<AdminQuery>,
        State(data): State<Api>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let all_users = get_users(db, user.to_option()).await;

        (
            StatusCode::OK,
            Json(
                all_users
                    .iter()
                    .filter_map(|user| user.get_email().ok())
                    .collect::<Vec<String>>(),
            ),
        )
            .into_response()
    }

    pub async fn get_name_list(
        Query(user): Query<AdminQuery>,
        State(data): State<Api>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let all_users = get_users(db, user.to_option()).await;

        (
            StatusCode::OK,
            Json(
                all_users
                    .iter()
                    .filter_map(|user| user.get_name().ok())
                    .collect::<Vec<String>>(),
            ),
        )
            .into_response()
    }

    pub async fn get_account_list(State(data): State<Api>) -> impl IntoResponse {
        let db = &data.db;
        (
            StatusCode::OK,
            Json(
                user_account::Entity::find()
                    .all(db)
                    .await
                    .unwrap_or_default()
                    .iter()
                    .map(|account| account.username.clone())
                    .collect::<Vec<String>>(),
            ),
        )
            .into_response()
    }

    async fn get_users(
        db: &DatabaseConnection,
        users: Option<AdminQuery>,
    ) -> Vec<MijnBussieInstance> {
        if let Some(user) = users {
            let instances = user.get_instance_name(db).await;
            let mut users = vec![];
            for instance in instances {
                match MijnBussieInstance::find_by_username(db, &instance).await {
                    Some(user) => users.push(user),
                    None => continue,
                };
            }
            users
        } else {
            MijnBussieInstance::get_all_users(db)
                .await
                .unwrap_or_default()
        }
    }
}
