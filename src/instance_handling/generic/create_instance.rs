pub mod post {
    use axum::{http::StatusCode, response::IntoResponse};

    use entity::user_account;
    use futures::TryFutureExt;
    use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};

    use crate::{
        GenResult,
        instance_handling::entity::{MijnBussieUser, UserDataModel},
        web::user::UserAccount,
    };

    pub async fn create_instance_and_attach(
        db: &DatabaseConnection,
        user: &UserAccount,
        instance: MijnBussieUser,
    ) -> GenResult<impl IntoResponse> {
        let response = match MijnBussieUser::create_and_insert_models(instance, db, None)
            .and_then(|data| async move { attach_user_to_instance(db, user, &data).await })
            .await
        {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        };
        Ok(response)
    }

    pub async fn attach_user_to_instance(
        db: &DatabaseConnection,
        account: &UserAccount,
        instance: &UserDataModel,
    ) -> GenResult<()> {
        let mut active_model = account.inner.clone().into_active_model();
        active_model.backend_user = Set(Some(instance.user_name.clone()));
        user_account::Entity::update(active_model).exec(db).await?;
        Ok(())
    }
}
