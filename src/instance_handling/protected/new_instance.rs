use crate::{Client, web::user::GetUser};

use super::*;

pub fn router() -> Router<AppState> {
    Router::new().route("/add_instance", post(create_instance_and_attach_protected))
}

pub async fn create_instance_and_attach_protected(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Json(instance): Json<MijnBussieInstance>,
) -> impl IntoResponse {
    let db = &data.db;

    let user_account = match auth_session.get_user() {
        Ok(user) => user,
        Err(err) => return err.into_response(),
    };
    let mut instance = instance.censor();
    instance.online_created = true;
    // If personeelsnummer already exists, don't create this instance
    if MijnBussieInstance::get_id_from_personeelsnummer(db, &instance.personeelsnummer)
        .await
        .ok()
        .is_some()
    {
        return StatusCode::CONFLICT.into_response();
    };
    match create_instance_and_attach(db, &user_account, instance).await {
        Ok(_) => StatusCode::OK,
        Err(_err) => StatusCode::INTERNAL_SERVER_ERROR,
    }
    .into_response()
}
