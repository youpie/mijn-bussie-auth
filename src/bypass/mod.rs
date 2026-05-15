use axum::Router;

use crate::web::api::Api;

mod change_password;
mod create_instance;

pub fn router() -> Router<Api> {
    Router::new()
        .merge(self::change_password::router())
        .merge(self::create_instance::router())
}
