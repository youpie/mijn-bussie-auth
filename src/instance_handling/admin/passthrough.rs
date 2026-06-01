use axum::extract::rejection::QueryRejection;

use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{request}", get(instance_get))
        .route("/{request}", post(instance_post))
        .route("/kuma/{request}", post(handle_kuma))
        .route("/refresh", post(refresh_instance))
}

pub async fn instance_get(
    State(data): State<AppState>,
    Path(request_type): Path<InstanceGetRequests>,
    Query(user): Query<AdminQuery>,
) -> GenResult<String> {
    let instance_name =
        AdminQuery::map_instance_query_result(user.get_instance_name(&data.db).await)?;
    Ok(instance_api::get_request(&data.client, &instance_name, request_type).await?)
}

pub async fn instance_post(
    State(data): State<AppState>,
    Path(request_type): Path<InstancePostRequests>,
    Query(user): Query<AdminQuery>,
) -> GenResult<String> {
    let db = &data.db;

    // If instance passthrough request is Delete, the user must first be unassigned as to prevent the database from removing the account (admin only)
    if request_type == InstancePostRequests::Delete
        && let Ok(user_account) = user.get_user_account(db, true).await
    {
        _ = detach_user_from_instance(db, &user_account).await;
    }

    let instance_name = AdminQuery::map_instance_query_result(user.get_instance_name(db).await)?;
    Ok(instance_api::post_request(&data.client, &instance_name, request_type).await?)
}

pub async fn refresh_instance(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> GenResult<String> {
    let instance_name =
        AdminQuery::map_instance_query_result(user.get_instance_name(&data.db).await).ok();

    Ok(instance_api::refresh_user(&data.client, instance_name.as_deref()).await?)
}

pub async fn handle_kuma(
    Path(request): Path<KumaRequest>,
    State(data): State<AppState>,
    user: Result<Query<AdminQuery>, QueryRejection>,
) -> GenResult<()> {
    let Query(user) = user.unwrap_or_default();
    let instance_name =
        AdminQuery::map_instance_query_result(user.get_instance_name(&data.db).await).ok();
    Ok(kuma_request(&data.client, instance_name.as_deref(), request).await?)
}
