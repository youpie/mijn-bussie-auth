use axum::Router;

use crate::{instance_handling, web::api::Api};

pub mod find;

pub fn admin_router() -> Router<Api> {
    Router::new()
        .merge(instance_handling::router::admin_router())
        .merge(self::find::router())
}
