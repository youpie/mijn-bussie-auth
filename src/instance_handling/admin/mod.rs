use entity::user_account;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Related};
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

    async fn get_instance_from_account(&self, db: &DatabaseConnection) -> Option<UserDataModel> {
        if let Some(account_name) = &self.account_name {
            user_account::Entity::find_related()
                .filter(user_account::Column::Username.eq(account_name))
                .one(db)
                .await
                .ok()
                .flatten()
        } else {
            None
        }
    }

    pub async fn get_instance_name(&self, db: &DatabaseConnection) -> Option<String> {
        let instance = self.get_instance_from_account(db).await;
        if let Some(instance_model) = instance {
            Some(instance_model.user_name.clone())
        } else {
            self.instance_name.clone()
        }
    }
}
