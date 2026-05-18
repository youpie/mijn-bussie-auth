use bcrypt::DEFAULT_COST;
use entity::user_account;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};

use super::*;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct PasswordChange {
    #[serde_as(as = "NoneAsEmptyString")]
    pub password: Option<String>,
}

pub async fn change_password(
    db: &DatabaseConnection,
    username: String,
    new_password: String,
) -> GenResult<()> {
    if new_password.is_empty() {
        return Err(AppError::UserError(AppErrorContext::new_user(
            "Password cannot be empty",
        )));
    }
    let user_model = user_account::Entity::find()
        .filter(user_account::Column::Username.eq(username))
        .one(db)
        .await?;
    if let Some(user_model) = user_model {
        let mut active_model = user_model.into_active_model();
        active_model.password_hash =
            Set(
                tokio::task::spawn_blocking(|| bcrypt::hash(new_password, DEFAULT_COST))
                    .await
                    .d()?
                    .d()?,
            );
        user_account::Entity::update(active_model)
            .validate()?
            .exec(db)
            .await?;
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
