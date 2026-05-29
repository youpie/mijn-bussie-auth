use sea_orm::EntityTrait;
use serde_json::Value;
use std::collections::HashMap;

use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get_instance", get(get_instance_data_admin))
        .route("/example", get(get_example_user))
        .route("/failed_instances", get(get_failed_users))
        .route("/add_instance", post(create_instance_admin))
        .route(
            "/change_instance_information",
            post(change_instance_password_admin),
        )
        .route("/assign_instance", post(assign_instance_to_account))
        .route("/unassign_instance", post(unassign_instance_to_account))
        .route("/update_properties", post(update_properties_admin))
}

pub async fn get_instance_data_admin(
    Query(user): Query<AdminQuery>,
    State(data): State<AppState>,
) -> GenResult<Json<MijnBussieInstance>> {
    let db = &data.db;
    let instance_name = AdminQuery::map_instance_query_result(user.get_instance_name(db).await)?;

    Ok(Json(
        MijnBussieInstance::find_by_username(db, &instance_name).await?,
    ))
}

pub async fn get_example_user() -> Json<MijnBussieInstance> {
    Json(MijnBussieInstance::default())
}

pub async fn get_failed_users(
    State(data): State<AppState>,
) -> GenResult<Json<HashMap<String, Value>>> {
    let db = &data.db;
    let instances = user_data::Entity::find().all(db).await?;
    let usernames: Vec<String> = instances
        .into_iter()
        .map(|instance| instance.user_name)
        .collect();
    let mut failed_hashmap = HashMap::new();
    for username in usernames {
        _ = instance_api::get_request(&username, instance_api::InstanceGetRequests::ExitCode)
            .await
            .ok()
            .map(|response| serde_json::from_str::<Value>(&response).ok())
            .flatten()
            .is_some_and(|exit_code| {
                if exit_code["ExitCode"] != "OK" {
                    failed_hashmap.insert(username, exit_code["ExitCode"].clone());
                }
                true
            });
    }
    Ok(Json(failed_hashmap))
}

pub async fn update_properties_admin(
    State(data): State<AppState>,
    Json(instance): Json<MijnBussieInstance>,
) -> GenResult<()> {
    let db = &data.db;
    Ok(instance.update_properties(db).await?)
}

pub async fn create_instance_admin(
    State(data): State<AppState>,
    Json(instance): Json<MijnBussieInstance>,
) -> GenResult<String> {
    let db = &data.db;
    Ok(
        MijnBussieInstance::create_and_insert_instance(instance, db, true)
            .await?
            .to_string(),
    )
}

pub async fn assign_instance_to_account(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> impl IntoResponse {
    let db = &data.db;
    if let Some(user_account) = user.get_user_account(db, false).await
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

pub async fn unassign_instance_to_account(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> impl IntoResponse {
    let db = &data.db;
    if let Some(user_account) = user.get_user_account(db, true).await {
        match remove_user_from_instance(db, &user_account).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub async fn change_instance_password_admin(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
    Json(password): Json<InstanceInformation>,
) -> impl IntoResponse {
    let db = &data.db;
    let instance = user.get_instance_from_query(db).await;
    if let Some(instance) = instance {
        match password.change_information(db, &instance).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    } else {
        (StatusCode::NO_CONTENT, format!("User {:?} not found", user)).into_response()
    }
}
