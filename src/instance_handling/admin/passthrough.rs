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
) -> impl IntoResponse {
    let db = &data.db;
    let instance_name =
        match AdminQuery::map_instance_query_result(user.get_instance_name(db).await) {
            Ok(name) => name,
            Err(names) => return names.into_response(),
        };
    let response = match instance_api::get_request(&instance_name, request_type).await {
        Ok(respone) => respone,
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    };
    response.into_response()
}

pub async fn instance_post(
    State(data): State<AppState>,
    Path(request_type): Path<InstancePostRequests>,
    Query(user): Query<AdminQuery>,
) -> impl IntoResponse {
    let db = &data.db;

    // If instance passthrough request is Delete, the user must first be unassigned as to prevent the database from removing the account (admin only)
    if request_type == InstancePostRequests::Delete {
        let user_account = user.get_user_account(db, true).await;
        if let Some(account) = user_account {
            _ = remove_user_from_instance(db, &account).await;
        }
    }

    let instance_name =
        match AdminQuery::map_instance_query_result(user.get_instance_name(db).await) {
            Ok(name) => name,
            Err(names) => return names.into_response(),
        };
    match instance_api::post_request(&instance_name, request_type).await {
        Ok(response) => response,
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
    .into_response()
}

pub async fn refresh_instance(
    State(data): State<AppState>,
    Query(user): Query<AdminQuery>,
) -> impl IntoResponse {
    let instance_name =
        match AdminQuery::map_instance_query_result(user.get_instance_name(&data.db).await) {
            Ok(name) => Some(name),
            Err(err) if err.0 == StatusCode::MULTIPLE_CHOICES => return err.into_response(),
            Err(_) => None,
        };

    match instance_api::refresh_user(instance_name.as_deref()).await {
        Ok(started) => started,
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
    .into_response()
}

#[axum::debug_handler]
pub async fn handle_kuma(
    Path(request): Path<KumaRequest>,
    State(data): State<AppState>,
    user: Result<Query<AdminQuery>, QueryRejection>,
) -> impl IntoResponse {
    let Query(user) = user.unwrap_or_default();
    let instance_name =
        match AdminQuery::map_instance_query_result(user.get_instance_name(&data.db).await) {
            Ok(name) => Some(name),
            Err(err) if err.0 == StatusCode::MULTIPLE_CHOICES => return err.into_response(),
            Err(_) => None,
        };
    match kuma_request(instance_name.as_deref(), request).await {
        Ok(code) => code.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
