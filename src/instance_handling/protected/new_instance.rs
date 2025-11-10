use crate::web::api::Api;
use axum::Router;
use axum::routing::post;

pub fn router() -> Router<Api> {
    Router::new().route(
        "/add_instance",
        post(self::post::create_instance_and_attach_protected),
    )
}

mod post {
    use axum::{Json, extract::State, response::IntoResponse};
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            entity::MijnBussieUser, generic::create_instance::post::create_instance_and_attach,
        },
        web::{
            api::Api,
            user::{AuthSession, GetUser},
        },
    };

    pub async fn create_instance_and_attach_protected(
        auth_session: AuthSession,
        State(data): State<Api>,
        Json(instance): Json<MijnBussieUser>,
    ) -> impl IntoResponse {
        let db = &data.db;

        let user_account = match auth_session.get_user() {
            Ok(user) => user,
            Err(err) => return err.into_response(),
        };

        // If personeelsnummer already exists, don't create this instance
        if MijnBussieUser::find_existing(db, &instance.personeelsnummer)
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
}
