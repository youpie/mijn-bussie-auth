use std::collections::HashSet;
use std::str::FromStr;

use axum::response::IntoResponse;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend};
use bcrypt::DEFAULT_COST;
use entity::{user_account, user_data};
use reqwest::StatusCode;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::ColumnTrait;
use sea_orm::ModelTrait;
use sea_orm::{DatabaseConnection, DerivePartialModel, EntityTrait, QueryFilter};
use serde::Deserialize;
use tokio::task;

use crate::GenResult;

#[derive(strum::EnumString, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Permissions {
    Admin,
    #[default]
    User,
}

#[derive(DerivePartialModel, Debug, Clone)]
#[sea_orm(entity = "user_account::Entity")]
pub struct UserAccount {
    #[sea_orm(nested)]
    pub inner: user_account::Model,
}

impl UserAccount {
    pub async fn add_user(db: &DatabaseConnection, creds: Credentials) -> GenResult<()> {
        let account = user_account::ActiveModel {
            account_id: NotSet,
            username: Set(creds.username),
            password_hash: Set(bcrypt::hash(creds.password, DEFAULT_COST)?),
            role: Set("User".to_owned()),
            backend_user: NotSet,
        };
        user_account::Entity::insert(account).exec(db).await?;
        Ok(())
    }

    pub async fn get_instance_data(
        &self,
        db: &DatabaseConnection,
    ) -> GenResult<Option<user_data::Model>> {
        Ok(self.inner.find_related(user_data::Entity).one(db).await?)
    }
}

impl AuthUser for UserAccount {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.inner.account_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.inner.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: DatabaseConnection,
}

impl Backend {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SeaORM(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Task(#[from] task::JoinError),

    #[error(transparent)]
    EnumParse(#[from] strum::ParseError),
}

type BackendResult<T> = Result<T, Error>;

impl AuthnBackend for Backend {
    type User = UserAccount;

    type Credentials = Credentials;

    type Error = Error;

    async fn authenticate(&self, creds: Self::Credentials) -> BackendResult<Option<Self::User>> {
        let user = user_account::Entity::find()
            .filter(user_account::Column::Username.contains(creds.username))
            .into_partial_model::<UserAccount>()
            .one(&self.db)
            .await?;

        let verified_account = task::spawn_blocking(|| {
            user.filter(|user| bcrypt::verify(creds.password, &user.inner.password_hash).is_ok())
        })
        .await?;
        Ok(verified_account)
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> BackendResult<Option<Self::User>> {
        let user = user_account::Entity::find_by_id(*user_id)
            .into_partial_model::<UserAccount>()
            .one(&self.db)
            .await?;
        Ok(user)
    }
}

impl AuthzBackend for Backend {
    type Permission = Permissions;

    async fn get_user_permissions(
        &self,
        _user: &Self::User,
    ) -> BackendResult<HashSet<Self::Permission>> {
        let mut hash_set = HashSet::new();
        hash_set.insert(Permissions::from_str(&_user.inner.role).unwrap_or_default());
        Ok(hash_set)
    }

    // No group permissions implemented (yet)
    async fn get_group_permissions(
        &self,
        _user: &Self::User,
    ) -> BackendResult<HashSet<Self::Permission>> {
        Ok(std::collections::HashSet::new())
    }

    async fn get_all_permissions(
        &self,
        user: &Self::User,
    ) -> BackendResult<HashSet<Self::Permission>> {
        Self::get_user_permissions(&self, user).await
    }

    async fn has_perm(&self, user: &Self::User, perm: Self::Permission) -> BackendResult<bool> {
        Ok(self.get_all_permissions(user).await?.contains(&perm))
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;

pub trait GetUser {
    fn get_user(self) -> Result<UserAccount, StatusCode>;
}

impl GetUser for AuthSession {
    fn get_user(self) -> Result<UserAccount, StatusCode> {
        match self.user {
            Some(user) => Ok(user),
            None => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
