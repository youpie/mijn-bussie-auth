use entity::user_account;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::web::user::UserAccount;
use sea_orm::ColumnTrait;

pub mod db;
pub mod instance_management;
pub mod passthrough;

#[derive(Deserialize)]
pub struct AdminQuery {
    pub user_name: String,
}

impl AdminQuery {
    pub async fn get_user_account(&self, db: &DatabaseConnection) -> Option<UserAccount> {
        user_account::Entity::find()
            .filter(user_account::Column::Username.eq(self.user_name.clone()))
            .into_partial_model::<UserAccount>()
            .one(db)
            .await
            .ok()
            .flatten()
    }
}
