use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{request}", get(get_instance))
        .route("/{request}", post(post_instance))
}

pub async fn get_instance(
    auth_session: AuthSession,
    Path(request_type): Path<InstanceGetRequests>,
) -> impl IntoResponse {
    let user = match auth_session.get_user() {
        Ok(user) => user,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match instance_api::get_request(&user.inner.backend_user.unwrap_or_default(), request_type)
        .await
    {
        Ok(link) if link.0 == StatusCode::OK => link.into_response(),
        Ok(link) => link.0.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn post_instance(
    auth_session: AuthSession,
    Path(request_type): Path<InstancePostRequests>,
) -> impl IntoResponse {
    let user = match auth_session.get_user() {
        Ok(user) => user,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match instance_api::post_request(&user.inner.backend_user.unwrap_or_default(), request_type)
        .await
    {
        Ok(link) if link.0 == StatusCode::OK => link.into_response(),
        Ok(link) => link.0.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
