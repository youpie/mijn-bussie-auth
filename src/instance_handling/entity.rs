use entity::{user_data, user_properties};
use sea_orm::ActiveValue::{NotSet, Set};
// type UserPropertiesModel = user_properties::Model;
use crate::{Client, GenResult, decrypt_value, encrypt_value};
use sea_orm::ActiveModelTrait;
use sea_orm::{ColumnTrait, IntoActiveModel};
use sea_orm::{DatabaseConnection, DerivePartialModel, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

pub type UserDataModel = user_data::Model;

const EMAIL_INTERVAL: i32 = 3*60;
const BARE_INTERVAL: i32 = 6*60;

/// Encrypted values:
/// * Personeelsnummer
/// * Password
/// * Email
/// * Name
///
/// Ignored values:
/// * file_name
/// * user_name (if not admin)
/// * All id's
///     From properties:
/// * execution_minute
/// * send_error_mail
/// * execution_interval_minutes
/// * auto_delete_account
#[derive(Debug, DerivePartialModel, Deserialize, Serialize, Clone, Default)]
#[sea_orm(entity = "entity::user_data::Entity")]
pub struct MijnBussieInstance {
    #[serde(default)]
    pub user_data_id: i32,
    #[serde(default)]
    pub user_name: String,
    #[serde(skip_serializing_if = "String::is_filled", default)]
    pub personeelsnummer: String,
    #[serde(skip_serializing_if = "String::is_filled", default)]
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_some", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "String::is_filled", default)]
    pub email: String,
    #[sea_orm(nested)]
    pub user_properties: user_properties::Model,
}

impl MijnBussieInstance {
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
        clone.email = decrypt_value(&self.email, false)?;
        // clone.name = decrypt_value(&self.name)?;
        clone.password = decrypt_value(&self.password, false)?;
        clone.personeelsnummer = decrypt_value(&self.personeelsnummer, false)?;
        Ok(clone)
    }

    pub fn get_name(&self) -> GenResult<String> {
        match &self.name {
            Some(name) => Ok(decrypt_value(name, false)?),
            None => Err("Empty name".into()),
        }
    }

    pub fn get_email(&self) -> GenResult<String> {
        Ok(decrypt_value(&self.email, false)?)
    }

    pub async fn create_and_insert_instance(
        self,
        db: &DatabaseConnection,
        custom_username: bool,
    ) -> GenResult<(UserDataModel, bool)> {
        // Remove leading 0's from username
        let user_name = if custom_username && !self.user_name.is_empty() {
            self.user_name.clone()
        } else {
            self.personeelsnummer.parse::<u64>()?.to_string()
        };
        let random_filename = random_str::get_string(12, true, true, true, false);

        let mut user_properties = self.user_properties.clone().into_active_model();
        user_properties.user_properties_id = NotSet;

        let user_data = user_data::ActiveModel {
            user_data_id: NotSet,
            user_name: Set(user_name),
            personeelsnummer: Set(encrypt_value(&self.personeelsnummer)?),
            password: Set(encrypt_value(&self.password)?),
            email: Set(encrypt_value(&self.email)?),
            file_name: Set(random_filename),
            user_properties: NotSet,
            custom_general_properties: NotSet,
            name: NotSet,
            last_execution_date: NotSet,
            last_succesfull_sign_in_date: NotSet,
            last_system_execution_date: NotSet,
            creation_date: Set(chrono::offset::Utc::now().naive_utc()),
        };
        match self.find_existing_instance(db).await {
            Some(matching_instance) => Ok((matching_instance, true)),
            None => Ok((
                Self::add_new_user_to_db(db, user_properties, user_data).await?,
                false,
            )),
        }
    }

    async fn add_new_user_to_db(
        db: &DatabaseConnection,
        user_properties: user_properties::ActiveModel,
        mut user_data: user_data::ActiveModel,
    ) -> GenResult<user_data::Model> {
        let res = user_properties::Entity::insert(user_properties)
            .exec(db)
            .await?;
        println!("id {}", res.last_insert_id);
        user_data.user_properties = Set(res.last_insert_id);

        let data_res = user_data::Entity::insert(user_data)
            .exec_with_returning(db)
            .await?;
        Ok(data_res)
    }

    pub async fn find_existing_instance(&self, db: &DatabaseConnection) -> Option<UserDataModel> {
        let existing_instances = user_data::Entity::find().all(db).await.ok()?;
        let matching_instance = existing_instances
            .iter()
            .find(|instance| instance.user_name == self.user_name);
        if let Some(instance_match) = matching_instance {
            println!(
                "An existing instance with the same username has been found, determining if actual match"
            );
            let match_email = decrypt_value(&instance_match.email, true).ok()?;
            let match_personeelsnummer =
                decrypt_value(&instance_match.personeelsnummer, true).ok()?;
            if match_email == self.email.to_lowercase()
                && match_personeelsnummer == self.personeelsnummer.to_lowercase()
            {
                println!("It is actually a match!");
                return Some(instance_match.to_owned());
            }
        }
        None
    }

    pub async fn update_properties(self, db: &DatabaseConnection) -> GenResult<()> {
        let properties = self.user_properties.into_active_model().reset_all();
        user_properties::Entity::update(properties)
            .validate()?
            .exec(db)
            .await?;
        Ok(())
    }

    pub fn calculate_execution_interval(&self) -> i32 {
        let properties = &self.user_properties;
        if properties.send_mail_new_shift | properties.send_mail_updated_shift | properties.send_mail_removed_shift {
            EMAIL_INTERVAL
        } else {
            BARE_INTERVAL
        }
    }
}


pub trait FindByUsername {
    async fn find_by_username(db: &DatabaseConnection, user_name: &str) -> Option<UserDataModel>;
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
}

impl Client for MijnBussieInstance {
    fn censor(self) -> Self {
        let mut empty_instance = MijnBussieInstance::default();

        let properties = &mut empty_instance.user_properties;

        properties.execution_minute = random_str::get_int(0, 59);
        properties.execution_interval_minutes = self.calculate_execution_interval();

        empty_instance.user_name = self.user_name;
        empty_instance.password = self.password;
        empty_instance.email = self.email;
        empty_instance.personeelsnummer = self.personeelsnummer;

        let properties_self = &self.user_properties;

        properties.send_mail_new_shift = properties_self.send_failed_signin_mail;
        properties.send_mail_removed_shift = properties_self.send_mail_removed_shift;
        properties.send_mail_updated_shift = properties_self.send_mail_updated_shift;
        properties.split_night_shift = properties_self.split_night_shift;
        properties.stop_midnight_shift = properties_self.stop_midnight_shift;

        properties.auto_delete_account = true;
        properties.send_welcome_mail = true;
        properties.user_properties_id = 0;
        
        empty_instance
    }
}

trait Filled {
    fn is_filled(&self) -> bool;
}

impl Filled for String {
    fn is_filled(&self) -> bool {
        !self.is_empty()
    }
}
