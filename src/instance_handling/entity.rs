use entity::{user_data, user_properties};
use sea_orm::ActiveValue::{NotSet, Set};
// type UserPropertiesModel = user_properties::Model;
use crate::{GenResult, add_new_user_to_db, decrypt_value, encrypt_value};
use sea_orm::ActiveModelTrait;
use sea_orm::{ColumnTrait, IntoActiveModel};
use sea_orm::{DatabaseConnection, DerivePartialModel, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

pub type UserDataModel = user_data::Model;

/// Encrypted values:
///
/// * Personeelsnummer
/// * Password
/// * Email
/// * Name
#[derive(Debug, DerivePartialModel, Deserialize, Serialize, Clone)]
#[sea_orm(entity = "entity::user_data::Entity")]
pub struct MijnBussieUser {
    #[serde(default)]
    pub user_data_id: i32,
    #[serde(default)]
    pub user_name: String,
    #[serde(skip_serializing, default)]
    pub personeelsnummer: String,
    #[serde(skip_serializing, default)]
    pub password: String,
    pub name: Option<String>,
    #[serde(skip_serializing, default)]
    pub email: String,
    #[sea_orm(nested)]
    pub user_properties: user_properties::Model,
}

impl MijnBussieUser {
    pub async fn get_id_from_personeelsnummer(
        db: &DatabaseConnection,
        personeelsnummer: &str,
    ) -> GenResult<Option<i32>> {
        let personeelsnummer_int = personeelsnummer.parse::<u64>()?.to_string();
        let user_exists = user_data::Entity::find()
            .filter(user_data::Column::UserName.contains(personeelsnummer_int))
            .one(db)
            .await?;
        Ok(user_exists.map(|model| model.user_data_id))
    }

    pub async fn find_by_username(db: &DatabaseConnection, user_name: &str) -> Option<Self> {
        user_data::Entity::find()
            .filter(user_data::Column::UserName.eq(user_name))
            .left_join(user_properties::Entity)
            .into_partial_model::<Self>()
            .one(db)
            .await
            .ok()
            .flatten()
    }

    pub async fn get_all_users(db: &DatabaseConnection) -> GenResult<Vec<Self>> {
        Ok(user_data::Entity::find()
            .left_join(user_properties::Entity)
            .into_partial_model::<Self>()
            .all(db)
            .await?)
    }

    /// **Wont deserialize name**
    pub fn _decrypt_values(&self) -> GenResult<Self> {
        let mut clone = self.clone();
        clone.email = decrypt_value(&self.email)?;
        // clone.name = decrypt_value(&self.name)?;
        clone.password = decrypt_value(&self.password)?;
        clone.personeelsnummer = decrypt_value(&self.personeelsnummer)?;
        Ok(clone)
    }

    pub fn get_name(&self) -> GenResult<String> {
        match &self.name {
            Some(name) => Ok(decrypt_value(name)?),
            None => Err("Empty name".into()),
        }
    }

    pub fn get_email(&self) -> GenResult<String> {
        Ok(decrypt_value(&self.email)?)
    }

    pub async fn create_and_insert_models(
        self,
        db: &DatabaseConnection,
        custom_username: bool,
    ) -> GenResult<UserDataModel> {
        // Remove leading 0's from
        let user_name = if custom_username && !self.user_name.is_empty() {
            self.user_name
        } else {
            self.personeelsnummer.parse::<u64>()?.to_string()
        };
        let execution_time = random_str::get_int(0, 59);
        let random_filename = random_str::get_string(12, true, true, true, false);

        let mut user_properties = self.user_properties.into_active_model();
        user_properties.execution_minute = Set(execution_time);

        let user_data = user_data::ActiveModel {
            user_data_id: NotSet,
            user_name: Set(user_name),
            personeelsnummer: Set(encrypt_value(self.personeelsnummer)?),
            password: Set(encrypt_value(self.password)?),
            email: Set(encrypt_value(self.email)?),
            file_name: Set(random_filename),
            user_properties: NotSet,
            custom_general_properties: NotSet,
            name: NotSet,
        };
        add_new_user_to_db(db, user_properties, user_data).await
    }

    pub async fn update_properties(self, db: &DatabaseConnection) -> GenResult<()> {
        let properties = self.user_properties.into_active_model().reset_all();
        user_properties::Entity::update(properties)
            .validate()?
            .exec(db)
            .await?;
        Ok(())
    }
}

pub trait FindByUsername {
    async fn find_by_username(db: &DatabaseConnection, user_name: &str) -> Option<UserDataModel>;
    async fn find_by_username_result(
        db: &DatabaseConnection,
        user_name: &str,
    ) -> GenResult<UserDataModel>;
}

impl FindByUsername for user_data::Model {
    async fn find_by_username(db: &DatabaseConnection, user_name: &str) -> Option<UserDataModel> {
        user_data::Entity::find()
            .filter(user_data::Column::UserName.eq(user_name))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_username_result(
        db: &DatabaseConnection,
        user_name: &str,
    ) -> GenResult<UserDataModel> {
        match Self::find_by_username(db, user_name).await {
            Some(value) => Ok(value),
            None => Err("None".into()),
        }
    }
}
