use axum::response::IntoResponse;
use entity::user_data;
use reqwest::StatusCode;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde::Deserialize;

use crate::{GenResult, encrypt_value, instance_handling::instance_api::Instance};

#[derive(Debug, Deserialize)]
pub struct InstanceInformation {
    password: Option<String>,
    email: Option<String>,
    personeelsnummer: Option<String>,
    username: Option<String>,
}

impl InstanceInformation {
    // Generic function for chaning user properties
    pub async fn change_information(
        &self,
        db: &DatabaseConnection,
        instance: &user_data::Model,
    ) -> GenResult<impl IntoResponse> {
        let user_name = instance.user_name.clone();
        let mut instance_data = instance.clone().into_active_model();
        if let Some(new_password) = &self.password {
            instance_data.password = Set(encrypt_value(&new_password)?);
        }
        if let Some(new_email) = &self.email {
            instance_data.email = Set(encrypt_value(&new_email)?);
        }
        if let Some(new_personeelsnummer) = &self.personeelsnummer {
            instance_data.email = Set(new_personeelsnummer.clone());
        }
        if let Some(new_username) = &self.username {
            instance_data.email = Set(new_username.clone());
        }

        user_data::Entity::update(instance_data).exec(db).await?;
        Instance::refresh_user(Some(&user_name)).await?;
        Ok(StatusCode::OK.into_response())
    }

    pub fn censor(mut self) -> Self {
        self.personeelsnummer = None;
        self.username = None;
        self
    }
}
