pub mod post {
    use axum::response::IntoResponse;
    use entity::user_data;
    use reqwest::StatusCode;
    use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};
    use serde::Deserialize;

    use crate::{
        GenResult, encode_password, instance_handling::instance_api::Instance,
        web::user::UserAccount,
    };

    #[derive(Deserialize)]
    pub struct PasswordChange {
        password: String,
    }

    pub async fn change_password(
        db: &DatabaseConnection,
        account: &UserAccount,
        new_password: &PasswordChange,
    ) -> GenResult<impl IntoResponse> {
        let response = if let Ok(Some(instance_data)) = account.get_instance_data(db).await {
            let user_name = instance_data.user_name.clone();
            let mut instance_data = instance_data.into_active_model();
            instance_data.password = Set(encode_password(new_password.password.clone()));
            user_data::Entity::update(instance_data).exec(db).await?;
            Instance::refresh_user(&user_name).await?;
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        Ok(response.into_response())
    }
}
