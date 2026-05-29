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
) -> GenResult<()> {
    let db = data.db;
    let user_instance = auth_session.get_user()?.get_instance_data(&db).await?;
    let information = information.censor();
    Ok(information.change_information(&db, &user_instance).await?)
}
