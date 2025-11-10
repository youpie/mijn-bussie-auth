use axum::Router;

use crate::{
    instance_handling::{
        admin::{db, instance_management, passthrough},
        protected::{change_password, new_instance},
    },
    web::api::Api,
};

pub fn protected_router() -> Router<Api> {
    Router::new()
        .merge(new_instance::router())
        .merge(change_password::router())
}

pub fn admin_router() -> Router<Api> {
    Router::new()
        .merge(db::router())
        .merge(instance_management::router())
        .merge(passthrough::router())
}
