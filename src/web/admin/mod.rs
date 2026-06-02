use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::instance_handling::{self, admin::AdminQuery};

use super::*;

pub mod account_handling;
pub mod find;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .merge(instance_handling::admin_router())
        .merge(self::find::router())
        .merge(self::account_handling::router())
}

async fn find_user_account(
    db: &DatabaseConnection,
    query: &AdminQuery,
) -> GenResult<user_account::Model> {
    if let Some(account_name) = query.account_name.as_ref() {
        Ok(user_account::Entity::find()
            .filter(user_account::Column::Username.eq(account_name))
            .one(db)
            .await?
            .result_reason("Account not found")?)
    } else {
        Err(AppError::NotFound)
    }
}
