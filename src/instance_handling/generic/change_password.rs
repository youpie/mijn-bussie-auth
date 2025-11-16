pub mod post {
    use axum::response::IntoResponse;
    use entity::user_data;
    use reqwest::StatusCode;
    use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};
    use serde::Deserialize;

    use crate::{GenResult, encrypt_value, instance_handling::instance_api::Instance};

    #[derive(Deserialize)]
    pub struct PasswordChange {
        password: String,
    }

    pub async fn change_password(
        db: &DatabaseConnection,
        instance: &user_data::Model,
        new_password: &PasswordChange,
    ) -> GenResult<impl IntoResponse> {
        let user_name = instance.user_name.clone();
        let mut instance_data = instance.clone().into_active_model();
        instance_data.password = Set(encrypt_value(new_password.password.clone())?);
        user_data::Entity::update(instance_data).exec(db).await?;
        Instance::refresh_user(Some(&user_name)).await?;
        Ok(StatusCode::OK.into_response())
    }
}
