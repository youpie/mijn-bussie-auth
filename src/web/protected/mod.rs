mod account_handling;
mod information;

use super::*;

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .merge(information::router())
        .merge(account_handling::router())
}
