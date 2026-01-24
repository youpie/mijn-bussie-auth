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
    use entity::{user_account, user_data};
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

    pub async fn get_account_list(
        State(data): State<Api>,
        Query(users): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let all_accounts = user_account::Entity::find()
            .find_with_related(user_data::Entity)
            .all(db)
            .await
            .unwrap_or_default();

        // If a user account has been specified. Print only that user
        let specific_user = users
            .get_user_account(db)
            .await
            .and_then(|account| Some(account.inner.username.clone()));
        let mut account_combination = vec![];
        for account in all_accounts {
            if specific_user.is_none() || Some(account.0.username.clone()) == specific_user {
                let linked_instance = account
                    .1
                    .first()
                    .and_then(|instance| Some(instance.user_name.clone()));
                account_combination.push((account.0.username, account.0.role, linked_instance));
            }
        }

        (StatusCode::OK, Json(account_combination)).into_response()
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
