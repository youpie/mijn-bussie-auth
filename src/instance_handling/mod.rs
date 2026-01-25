pub mod admin;
pub mod entity;
pub mod generic;
mod instance_api;
mod protected;

use axum::Router;

use crate::{
    instance_handling::{
        admin::{db, instance_management, passthrough},
        protected::{change_information, new_instance},
    },
    web::api::Api,
};

pub fn protected_router() -> Router<Api> {
    Router::new()
        .merge(new_instance::router())
        .merge(change_information::router())
        .merge(protected::passthrough::router())
}

pub fn admin_router() -> Router<Api> {
    Router::new()
        .merge(db::router())
        .merge(instance_management::router())
        .merge(passthrough::router())
}
