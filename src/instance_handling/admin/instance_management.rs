use crate::web::api::Api;
use axum::Router;
use axum::routing::post;

pub fn router() -> Router<Api> {
    Router::new().route("/add_instance", post(self::post::create_instance_admin))
}

mod post {
    use axum::{
        Json,
        extract::{Query, State},
        response::IntoResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        instance_handling::{
            admin::AdminQuery, entity::MijnBussieUser,
            generic::create_instance::post::create_instance_and_attach,
        },
        web::api::Api,
    };

    pub async fn create_instance_admin(
        State(data): State<Api>,
        Query(user): Query<AdminQuery>,
        Json(instance): Json<MijnBussieUser>,
    ) -> impl IntoResponse {
        let db = &data.db;
        let user = match user.get_user_account(db).await {
            Some(user) => user,
            None => {
                return (StatusCode::NOT_FOUND, "User not found").into_response();
            }
        };
        match create_instance_and_attach(db, &user, instance).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
