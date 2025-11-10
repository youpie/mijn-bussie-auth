use entity::{user_data, user_properties};
use sea_orm::ActiveValue::{NotSet, Set};
// type UserPropertiesModel = user_properties::Model;
use crate::{GenResult, add_user_to_db, encode_password};
use sea_orm::{ColumnTrait, IntoActiveModel};
use sea_orm::{DatabaseConnection, DerivePartialModel, EntityTrait, QueryFilter};
use serde::Deserialize;

pub type UserDataModel = user_data::Model;

#[derive(Debug, DerivePartialModel, Deserialize)]
#[sea_orm(entity = "entity::user_data::Entity")]
pub struct MijnBussieUser {
    pub personeelsnummer: String,
    pub password: String,
    pub email: String,
    #[sea_orm(nested)]
    pub user_properties: user_properties::Model,
}

impl MijnBussieUser {
    pub async fn find_existing(
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

    pub async fn _find_by_username(
        db: &DatabaseConnection,
        user_name: &str,
    ) -> Option<UserDataModel> {
        user_data::Entity::find()
            .filter(user_data::Column::UserName.eq(user_name))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    pub async fn create_and_insert_models(
        self,
        db: &DatabaseConnection,
        user_name: Option<String>,
    ) -> GenResult<UserDataModel> {
        // Remove leading 0's from
        let user_name = user_name.unwrap_or(self.personeelsnummer.parse::<u64>()?.to_string());
        let execution_time = random_str::get_int(0, 59);
        let random_filename = random_str::get_string(12, true, true, true, false);

        let mut user_properties = self.user_properties.into_active_model();
        user_properties.execution_minute = Set(execution_time);

        let user_data = user_data::ActiveModel {
            user_data_id: NotSet,
            user_name: Set(user_name),
            personeelsnummer: Set(encode_password(self.personeelsnummer)),
            password: Set(encode_password(self.password)),
            email: Set(encode_password(self.email)),
            file_name: Set(random_filename),
            user_properties: NotSet,
            custom_general_properties: NotSet,
        };
        add_user_to_db(db, user_properties, user_data).await
    }
}
