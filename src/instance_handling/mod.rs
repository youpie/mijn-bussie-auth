pub mod admin;
pub mod bypass;
pub mod entity;
pub mod generic;
pub mod instance_api;
mod protected;

use crate::prelude::*;

use self::entity::*;
use self::instance_api::*;
type UserDataModel = user_data::Model;
use crate::Client;

use self::generic::*;

pub fn protected_router() -> Router<AppState> {
    use self::protected::*;
    Router::new()
        .merge(new_instance::router())
        .merge(change_information::router())
        .merge(protected::passthrough::router())
}

pub fn admin_router() -> Router<AppState> {
    use self::admin::*;
    Router::new()
        .merge(db::router())
        .merge(instance_management::router())
        .merge(passthrough::router())
}
