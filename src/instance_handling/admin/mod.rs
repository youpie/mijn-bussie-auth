use entity::{user_account, user_data};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::{instance_handling::entity::UserDataModel, web::user::UserAccount};
use sea_orm::ColumnTrait;

pub mod db;
pub mod instance_management;
pub mod passthrough;

#[derive(Deserialize)]
pub struct AdminQuery {
    pub account_name: Option<String>,
    pub instance_name: Option<String>,
}

impl AdminQuery {
    pub async fn get_user_account(&self, db: &DatabaseConnection) -> Option<UserAccount> {
        if let Some(account_name) = &self.account_name {
            user_account::Entity::find()
                .filter(user_account::Column::Username.eq(account_name.clone()))
                .into_partial_model::<UserAccount>()
                .one(db)
                .await
                .ok()
                .flatten()
        } else {
            None
        }
    }

    pub async fn get_user_instance(&self, db: &DatabaseConnection) -> Option<UserDataModel> {
        if let Some(instance_name) = &self.instance_name {
            user_data::Entity::find()
                .filter(user_data::Column::UserName.eq(instance_name.clone()))
                .one(db)
                .await
                .ok()
                .flatten()
        } else {
            None
        }
    }
}
