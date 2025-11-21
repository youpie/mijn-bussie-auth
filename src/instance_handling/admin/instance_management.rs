use crate::web::api::Api;
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router<Api> {
    Router::new()
        .route("/get_instance", get(self::get::get_instance_data_admin))
        .route("/example", get(self::get::get_example_user))
        .route("/failed_instances", get(self::get::get_failed_users))
        .route("/add_instance", post(self::post::create_instance_admin))
        .route(
            "/change_instance_password",
            post(self::post::change_instance_password_admin),
        )
        .route(
            "/assign_instance",
            post(self::post::assign_instance_to_account),
        )
        .route(
            "/update_properties",
            post(self::post::update_properties_admin),
        )
}

mod get {
    use std::collections::HashMap;

    use axum::{
        Json,
        extract::{Query, State},
        response::IntoResponse,
    };
    use entity::user_data;
    use hyper::header;
    use reqwest::StatusCode;
    use sea_orm::EntityTrait;
    use serde::Deserialize;

    use crate::{
        instance_handling::{admin::AdminQuery, entity::MijnBussieUser, instance_api},
        web::api::Api,
    };

    pub async fn get_instance_data_admin(
        Query(user): Query<AdminQuery>,
        State(data): State<Api>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let instance_name =
            match AdminQuery::map_instance_query_result(user.get_instance_name(db).await) {
                Ok(name) => name,
                Err(names) => return names.into_response(),
            };

        match MijnBussieUser::find_by_username(db, &instance_name).await {
            Some(instance_data) => (
                StatusCode::OK,
                serde_json::to_string_pretty(&instance_data).unwrap(),
            )
                .into_response(),
            None => (
                StatusCode::NO_CONTENT,
                format!("Could not find user_data from \"{instance_name}\""),
            )
                .into_response(),
        }
    }

    pub async fn get_example_user() -> impl IntoResponse {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            Json(MijnBussieUser::default()),
        )
            .into_response()
    }

    pub async fn get_failed_users(State(data): State<Api>) -> impl IntoResponse {
        let db = &data.db;
        if let Ok(instances) = user_data::Entity::find().all(db).await {
            let usernames: Vec<String> = instances
                .into_iter()
                .map(|instance| instance.user_name)
                .collect();
            let mut failed_hashmap = HashMap::new();
            for username in usernames {
                _ = instance_api::Instance::get_request(
                    &username,
                    instance_api::InstanceGetRequests::ExitCode,
                )
                .await
                .ok()
                .map(|response| response.1)
                .is_some_and(|exit_code| {
                    if exit_code != "OK" {
                        failed_hashmap.insert(username, exit_code);
                    }
                    true
                });
            }
            (StatusCode::OK, Json(failed_hashmap)).into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
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
            entity::{FindByUsername, MijnBussieUser, UserDataModel},
            generic::{
                change_password::post::{PasswordChange, change_password},
                create_instance::post::attach_user_to_instance,
            },
        },
        web::api::Api,
    };

    pub async fn update_properties_admin(
        State(data): State<Api>,
        Json(instance): Json<MijnBussieUser>,
    ) -> impl IntoResponse {
        let db = &data.db;
        match instance.update_properties(db).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }

    pub async fn create_instance_admin(
        State(data): State<Api>,
        Json(instance): Json<MijnBussieUser>,
    ) -> impl IntoResponse {
        let db = &data.db;
        match MijnBussieUser::create_and_insert_models(instance, db, true).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }

    pub async fn assign_instance_to_account(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
    ) -> impl IntoResponse {
        let db = &data.db;
        if let Some(user_account) = user.get_user_account(db).await
            && let Some(instance_name) = user.instance_name
            && let Some(instance_data) = UserDataModel::find_by_username(db, &instance_name).await
        {
            match attach_user_to_instance(db, &user_account, &instance_data).await {
                Ok(_) => StatusCode::OK.into_response(),
                Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            }
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }

    pub async fn change_instance_password_admin(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
        Json(password): Json<PasswordChange>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let instance = user.get_instance_from_query(db).await;
        if let Some(instance) = instance {
            match change_password(db, &instance, &password).await {
                Ok(_) => StatusCode::OK.into_response(),
                Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            }
        } else {
            (StatusCode::NO_CONTENT, format!("User {:?} not found", user)).into_response()
        }
    }
}
