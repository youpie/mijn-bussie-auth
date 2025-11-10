use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use dotenvy::var;
use sea_orm::{Database, DatabaseConnection, sqlx::PgPool};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};
use tower_sessions_sqlx_store::PostgresStore;

use crate::{
    GenResult,
    web::{auth, user::Backend},
};

pub struct Api {
    db: DatabaseConnection,
}

impl Api {
    pub async fn new() -> GenResult<Self> {
        let db = Database::connect(&var("DATABASE_URL")?)
            .await
            .expect("Could not connect to database");

        Ok(Self { db })
    }

    pub async fn serve(self) -> GenResult<()> {
        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let pg_pool = PgPool::connect_lazy(&var("DATABASE_URL")?)?;
        let session_store = PostgresStore::new(pg_pool);
        session_store.migrate().await?;
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(self.db);
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let app = Router::new().merge(auth::router()).layer(auth_layer);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }
}
