use reqwest::StatusCode;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde::Deserialize;

use crate::{Client, encrypt_value};

use super::*;

#[derive(Debug, Deserialize, Default)]
pub struct InstanceInformation {
    password: Option<String>,
    email: Option<String>,
    personeelsnummer: Option<String>,
    user_name: Option<String>,
}

impl InstanceInformation {
    pub async fn change_information_protected(
        self,
        db: DatabaseConnection,
        user: UserAccount,
    ) -> impl IntoResponse {
        let response = if let Ok(Some(instance_data)) = user.get_instance_data(&db).await {
            match self.change_information(&db, &instance_data).await {
                Ok(response) => response.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        } else {
            StatusCode::NO_CONTENT.into_response()
        };
        response
    }

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
            instance_data.personeelsnummer = Set(new_personeelsnummer.clone());
        }
        if let Some(new_username) = &self.user_name {
            instance_data.user_name = Set(new_username.clone());
        }

        user_data::Entity::update(instance_data).exec(db).await?;
        refresh_user(Some(&user_name)).await?;
        Ok(StatusCode::OK.into_response())
    }
}

impl Client for InstanceInformation {
    fn censor(self) -> Self {
        let mut base = Self::default();
        base.email = self.email;
        base.password = self.password;
        base
    }
}
