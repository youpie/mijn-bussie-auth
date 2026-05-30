mod change_password;
mod create_instance;

use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(self::change_password::router())
        .merge(self::create_instance::router())
}
