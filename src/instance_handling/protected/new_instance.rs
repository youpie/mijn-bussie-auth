use crate::{Client, web::user::GetUser};

use super::*;

pub fn router() -> Router<AppState> {
    Router::new().route("/add_instance", post(create_instance_and_attach_protected))
}

pub async fn create_instance_and_attach_protected(
    auth_session: AuthSession,
    State(data): State<AppState>,
    Json(instance): Json<MijnBussieInstance>,
) -> GenResult<()> {
    let db = &data.db;

    let user_account = auth_session.get_user()?;
    let mut instance = instance.censor();
    instance.online_created = true;
    // If personeelsnummer already exists, don't create this instance
    match MijnBussieInstance::get_id_from_personeelsnummer(db, &instance.personeelsnummer).await {
        Err(e) => Err(e),
        Ok(_) => Err(AppError::AlreadyExists),
    }?;
    Ok(create_instance_and_attach(db, &user_account, instance).await?)
}
