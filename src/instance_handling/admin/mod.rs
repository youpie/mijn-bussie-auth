use axum::Json;
use entity::{user_account, user_data};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Related};
use serde::Deserialize;

use crate::{
    decrypt_value,
    instance_handling::entity::{FindByUsername, UserDataModel},
    web::user::UserAccount,
};
use sea_orm::ColumnTrait;

pub mod db;
pub mod instance_management;
pub mod passthrough;

#[derive(Deserialize, Debug)]
pub struct AdminQuery {
    pub account_name: Option<String>,
    pub instance_name: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
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

    async fn get_instance_from_account_name(
        &self,
        db: &DatabaseConnection,
    ) -> Option<UserDataModel> {
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

    // Multiple instances can have the same email, so a vec should be returned
    pub async fn get_instance_from_email(
        &self,
        db: &DatabaseConnection,
    ) -> Option<Vec<UserDataModel>> {
        user_data::Entity::find()
            .all(db)
            .await
            .ok()
            .and_then(|values| {
                Some(
                    values
                        .into_iter()
                        .filter(|adres| {
                            Some(decrypt_value(&adres.email).ok().as_ref())
                                .eq(&Some(self.email.as_ref()))
                        })
                        .collect(),
                )
            })
    }

    // Multiple instances can have the same email, so a vec should be returned
    pub async fn get_instance_from_name(
        &self,
        db: &DatabaseConnection,
    ) -> Option<Vec<UserDataModel>> {
        user_data::Entity::find()
            .all(db)
            .await
            .ok()
            .and_then(|values| {
                Some(
                    values
                        .into_iter()
                        .filter(|adres| {
                            Some(
                                &adres
                                    .name
                                    .as_ref()
                                    .and_then(|name| decrypt_value(name).ok())
                                    .as_ref(),
                            )
                            .eq(&Some(&self.name.as_ref()))
                        })
                        .collect(),
                )
            })
    }

    pub async fn get_instance_name(&self, db: &DatabaseConnection) -> Vec<String> {
        let instance = self.get_instance_from_account_name(db).await;
        if let Some(instance_model) = instance {
            vec![instance_model.user_name.clone()]
        } else if let Some(email_accounts) = self.get_instance_from_email(db).await {
            let email_users = email_accounts
                .iter()
                .map(|user| user.user_name.clone())
                .collect();
            email_users
        } else if let Some(name_accounts) = self.get_instance_from_name(db).await {
            let name_users = name_accounts
                .iter()
                .map(|user| user.user_name.clone())
                .collect();
            name_users
        } else {
            self.instance_name
                .clone()
                .map_or(vec![], |users| vec![users])
        }
    }

    pub fn map_instance_query_result(
        mut names: Vec<String>,
    ) -> Result<String, (StatusCode, Json<Vec<String>>)> {
        if names.len() == 1
            && let Some(instance_name) = names.pop()
        {
            Ok(instance_name)
        } else if names.is_empty() {
            Err((StatusCode::NOT_FOUND, Json(names)))
        } else {
            Err((StatusCode::CONFLICT, Json(names)))
        }
    }

    pub async fn get_instance_from_query(&self, db: &DatabaseConnection) -> Option<UserDataModel> {
        let instance_name = match Self::map_instance_query_result(self.get_instance_name(db).await)
        {
            Ok(name) => name,
            Err(_) => {
                return None;
            }
        };
        UserDataModel::find_by_username(db, &instance_name).await
    }
}
