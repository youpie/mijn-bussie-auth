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
        .route("/unassign_instance", post(unassign_instance_from_account))
        .route("/update_properties", post(update_properties_admin))
}

pub async fn get_instance_data_admin(
    Query(user): Query<AdminQuery>,
    State(data): State<AppState>,
) -> GenResult<Json<MijnBussieInstance>> {
    let db = &data.db;
    let instance_name = AdminQuery::map_instance_query_result(user.get_instance_name(db).await)?;
    let mut instance = MijnBussieInstance::find_by_username(db, &instance_name).await?;
    // TODO dit moet netjeser kunnen
    instance.password = String::new();
    instance.email = String::new();
    instance.name = instance.name.map(|_| String::new());
    instance.personeelsnummer = String::new();
    Ok(Json(instance))
}

pub async fn get_example_user() -> Json<MijnBussieInstance> {
    Json(MijnBussieInstance::default())
}

pub async fn get_failed_users(
    State(data): State<AppState>,
) -> GenResult<Json<HashMap<String, Value>>> {
    let instances = user_data::Entity::find().all(&data.db).await?;
    let usernames: Vec<String> = instances
        .into_iter()
        .map(|instance| instance.user_name)
        .collect();
    let mut failed_hashmap = HashMap::new();
    for username in usernames {
        _ = instance_api::get_request(
            &data.client,
            &username,
            instance_api::InstanceGetRequests::ExitCode,
        )
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
    Json(properties): Json<user_properties::Model>,
) -> GenResult<()> {
    let mut instance = MijnBussieInstance::default();
    instance.user_properties = properties;

    let db = &data.db;
    Ok(instance.update_properties(db).await?)
}

pub async fn create_instance_admin(
    State(data): State<AppState>,
    Json(instance): Json<MijnBussieInstance>,
) -> GenResult<(StatusCode, String)> {
    let db = &data.db;
    match MijnBussieInstance::create_and_insert_instance(instance, db, true).await? {
        InstanceMatchReturn::NewUser(user) => Ok((
            StatusCode::CREATED,
            InstanceMatchReturn::NewUser(user).to_string(),
        )),
        matching => Ok((StatusCode::CONFLICT, matching.to_string())),
    }
}

pub async fn assign_instance_to_account(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> GenResult<()> {
    let db = &data.db;
    if let Some(ref instance_name) = user.instance_name {
        let user_account = user.get_user_account(db, false).await?;
        let instance_data = UserDataModel::find_by_username(db, instance_name).await?;
        Ok(attach_user_to_instance(db, &user_account, &instance_data).await?)
    } else {
        Err(AppError::UserError(AppErrorContext::new_user(
            "Please set instance name!".to_owned(),
        )))
    }
}

pub async fn unassign_instance_from_account(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> GenResult<()> {
    let db = &data.db;
    let user_account = user.get_user_account(db, true).await?;
    Ok(detach_user_from_instance(db, &user_account).await?)
}

pub async fn change_instance_password_admin(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
    Json(password): Json<InstanceInformation>,
) -> GenResult<()> {
    let instance = user.get_instance_from_query(&data.db).await?;
    Ok(password.change_information(&data, &instance).await?)
}
