use axum::Router;

use crate::web::api::Api;

mod account_handling;
mod information;

pub fn protected_router() -> Router<Api> {
    Router::new()
        .merge(information::router())
        .merge(account_handling::router())
}
