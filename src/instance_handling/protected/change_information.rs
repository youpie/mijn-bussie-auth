use crate::Client;

pub use super::*;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/change_instance_information",
        post(change_information_protected),
    )
}

pub async fn change_information_protected(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Json(information): Json<InstanceInformation>,
) -> impl IntoResponse {
    let user = auth_session.user.expect("No user in protected space");
    let data = data.db;
    let information = information.censor();
    information.change_information_protected(data, user).await
}
