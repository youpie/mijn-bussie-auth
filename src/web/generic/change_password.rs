use bcrypt::DEFAULT_COST;
use entity::user_account;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;

use crate::GenResult;

#[derive(Debug, Deserialize)]
pub struct PasswordChange {
    pub password: String,
}

pub async fn change_password(
    db: &DatabaseConnection,
    username: String,
    new_password: String,
) -> GenResult<()> {
    let user_model = user_account::Entity::find()
        .filter(user_account::Column::Username.eq(username))
        .one(db)
        .await?;
    if let Some(user_model) = user_model {
        let mut active_model = user_model.into_active_model();
        active_model.password_hash =
            Set(tokio::task::spawn_blocking(|| bcrypt::hash(new_password, DEFAULT_COST)).await??);
        user_account::Entity::update(active_model)
            .validate()?
            .exec(db)
            .await?;
        Ok(())
    } else {
        Err("User not found".into())
    }
}
