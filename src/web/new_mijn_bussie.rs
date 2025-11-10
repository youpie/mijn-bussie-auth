use crate::{GenResult, web::api::Api};
use crate::{add_user_to_db, encode_password};
use axum::Router;
use axum::routing::post;
use entity::{user_data, user_properties};
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{ColumnTrait, IntoActiveModel};
use sea_orm::{DatabaseConnection, DerivePartialModel, EntityTrait, QueryFilter};
use serde::Deserialize;

// type UserPropertiesModel = user_properties::Model;
type UserDataModel = user_data::Model;

#[derive(Debug, DerivePartialModel, Deserialize)]
#[sea_orm(entity = "entity::user_data::Entity")]
struct MijnBussieUser {
    personeelsnummer: String,
    password: String,
    email: String,
    #[sea_orm(nested)]
    user_properties: user_properties::Model,
}

impl MijnBussieUser {
    async fn find_existing(
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

    async fn _find_by_username(db: &DatabaseConnection, user_name: &str) -> Option<UserDataModel> {
        user_data::Entity::find()
            .filter(user_data::Column::UserName.eq(user_name))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn create_and_insert_models(
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
            personeelsnummer: Set(self.personeelsnummer),
            password: Set(encode_password(self.password)),
            email: Set(self.email),
            file_name: Set(random_filename),
            user_properties: NotSet,
            custom_general_properties: NotSet,
        };
        add_user_to_db(db, user_properties, user_data).await
    }
}

pub fn router() -> Router<Api> {
    Router::new().route(
        "/add_instance",
        post(self::post::create_instance_and_attach),
    )
}

mod post {
    use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

    use entity::user_account;
    use futures::TryFutureExt;
    use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};

    use crate::{
        GenResult,
        web::{
            api::Api,
            new_mijn_bussie::{MijnBussieUser, UserDataModel},
            user::AuthSession,
        },
    };

    // #[derive(Deserialize)]
    // struct AdminQuery {
    //     account: String,
    //     instance_name: String,
    // }

    // pub async fn attach_instance(Query(info): Query<AdminQuery>, State(data): State<Api>) -> impl IntoResponse {
    //     match || async {
    //         let db = &data.db;
    //         let existing_instance = MijnBussieUser::find_by_username(db, &info.account).await;
    //         let user
    //     }().await {

    //     }
    // }

    pub async fn create_instance_and_attach(
        State(data): State<Api>,
        auth_session: AuthSession,
        Json(instance): Json<MijnBussieUser>,
    ) -> impl IntoResponse {
        let db = &data.db;

        // If personeelsnummer already exists, don't create this instance
        if MijnBussieUser::find_existing(db, &instance.personeelsnummer)
            .await
            .ok()
            .is_some()
        {
            return StatusCode::CONFLICT.into_response();
        }
        let db = &data.db;
        match MijnBussieUser::create_and_insert_models(instance, db, None)
            .and_then(|data| async move { attach_user_to_instance(db, &auth_session, &data).await })
            .await
        {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }

    async fn attach_user_to_instance(
        db: &DatabaseConnection,
        auth_session: &AuthSession,
        instance: &UserDataModel,
    ) -> GenResult<()> {
        if let Some(session_user) = &auth_session.user {
            let mut active_model = session_user.inner.clone().into_active_model();
            active_model.backend_user = Set(Some(instance.user_name.clone()));
            user_account::Entity::update(active_model).exec(db).await?;
            return Ok(());
        }
        Err("No session user set".into())
    }
}
