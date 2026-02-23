use axum::Router;

use crate::web::api::Api;

mod change_password;

pub fn router() -> Router<Api> {
    Router::new().merge(self::change_password::router())
}