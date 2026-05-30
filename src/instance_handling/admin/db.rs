use super::*;

pub fn router() -> Router<AppState> {
    Router::new().route("/instances", get(get_all_instances))
}

pub async fn get_all_instances(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> GenResult<Json<Vec<String>>> {
    let db = &data.db;
    let instances = user_data::Entity::find().all(db).await?;
    let specific_user = user.get_instance_name(db).await;
    let instances: Vec<String> = instances
        .iter()
        .filter_map(|x| {
            if specific_user.is_empty() || specific_user.contains(&x.user_name) {
                Some(x.user_name.to_owned())
            } else {
                None
            }
        })
        .collect();

    instances.not_found().map(|i| Json(i))
}
