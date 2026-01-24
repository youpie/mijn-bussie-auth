use axum::Router;
use sea_orm::DatabaseConnection;

use crate::{instance_handling, web::api::Api};

pub mod account_handling;
pub mod find;

pub fn admin_router() -> Router<Api> {
    Router::new()
        .merge(instance_handling::router::admin_router())
        .merge(self::find::router())
        .merge(self::account_handling::router())
}

fn find_user_account(db: &DatabaseConnection, query: AdminQuery)