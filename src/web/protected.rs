use axum::Router;
use axum::routing::{get, post};

use crate::web::api::Api;
use crate::web::new_mijn_bussie;

pub fn router() -> Router<Api> {
    Router::new()
        .route("/me", get(self::get::protected))
        .route("/change_password", post(self::post::change_password))
        .merge(new_mijn_bussie::router())
}

mod post {
    use std::str::FromStr;

    use axum::{Json, extract::State, response::IntoResponse};
    use dotenvy::var;
    use entity::user_data;
    use reqwest::{StatusCode, Url};
    use sea_orm::{ActiveValue::Set, EntityTrait, IntoActiveModel};
    use serde::Deserialize;

    use crate::{
        encode_password,
        web::{api::Api, user::AuthSession},
    };

    #[derive(Deserialize)]
    pub struct PasswordChange {
        password: String,
    }

    pub async fn change_password(
        auth_session: AuthSession,
        State(data): State<Api>,
        Json(new_password): Json<PasswordChange>,
    ) -> impl IntoResponse {
        let db = &data.db;
        if let Some(user_account) = auth_session.user
            && let Ok(Some(instance_data)) = user_account.get_instance_data(db).await
        {
            let mut instance_data = instance_data.into_active_model();
            instance_data.password = Set(encode_password(new_password.password));
            user_data::Entity::update(instance_data)
                .exec(db)
                .await
                .unwrap();
            let api_key = var("API_KEY").unwrap();
            let mut url = Url::from_str(&var("MIJN_BUSSIE_URL").unwrap()).unwrap();
            url.join("api/index/").unwrap();
            url.join(&user_account.inner.backend_user.unwrap()).unwrap();
            url.set_query(Some(&format!("key={api_key}")));
            reqwest::get(url).await.unwrap();
            StatusCode::OK.into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

mod get {
    use crate::web::user::AuthSession;
    use axum::response::IntoResponse;
    use reqwest::StatusCode;

    pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => (StatusCode::OK, user.inner.username).into_response(),

            None => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}
