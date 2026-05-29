use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{request}", get(generic_instance_get))
        .route("/{request}", post(generic_instance_post))
}

pub async fn generic_instance_get(
    auth_session: AuthSession,
    Path(request_type): Path<InstanceGetRequests>,
) -> GenResult<String> {
    let user = auth_session.get_user()?;
    Ok(
        instance_api::get_request(&user.inner.backend_user.unwrap_or_default(), request_type)
            .await?,
    )
}

pub async fn generic_instance_post(
    auth_session: AuthSession,
    Path(request_type): Path<InstancePostRequests>,
) -> GenResult<String> {
    let user = auth_session.get_user()?;
    Ok(
        instance_api::post_request(&user.inner.backend_user.unwrap_or_default(), request_type)
            .await?,
    )
}
