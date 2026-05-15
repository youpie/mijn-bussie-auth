pub mod post {
    use entity::user_account;
    use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};

    use crate::{
        GenResult,
        instance_handling::entity::{InstanceMatchReturn, MijnBussieInstance, UserDataModel},
        web::user::UserAccount,
    };

    pub async fn create_instance_and_attach(
        db: &DatabaseConnection,
        user: &UserAccount,
        instance: MijnBussieInstance,
    ) -> GenResult<()> {
        let instance =
            match MijnBussieInstance::create_and_insert_instance(instance, db, false).await? {
                InstanceMatchReturn::Exact(instance) => Some(instance),
                InstanceMatchReturn::NewUser(instance) => Some(instance),
                _ => None,
            };
        if let Some(instance) = instance {
            attach_user_to_instance(db, user, &instance).await?
        }
        Ok(())
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

    pub async fn remove_user_from_instance(
        db: &DatabaseConnection,
        account: &UserAccount,
    ) -> GenResult<()> {
        let mut active_model = account.inner.clone().into_active_model();
        active_model.backend_user = Set(None);
        user_account::Entity::update(active_model).exec(db).await?;
        Ok(())
    }
}
