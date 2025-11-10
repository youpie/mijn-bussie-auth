use axum::Router;

use crate::{
    instance_handling::protected::{change_password, new_instance},
    web::api::Api,
};

pub fn protected_router() -> Router<Api> {
    Router::new()
        .merge(new_instance::router())
        .merge(change_password::router())
}
