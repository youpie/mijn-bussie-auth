use std::{path::PathBuf, str::FromStr};

use axum::Router;
use axum_login::{AuthManagerLayerBuilder, login_required, permission_required};
use axum_server::tls_rustls::RustlsConfig;
use dotenvy::var;
use sea_orm::{Database, DatabaseConnection, sqlx::PgPool};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};
use tower_sessions_sqlx_store::PostgresStore;

use crate::{
    GenResult, instance_handling,
    web::{
        auth, new_user,
        user::{Backend, Permissions},
    },
};

#[derive(Debug, Clone)]
pub struct Api {
    pub db: DatabaseConnection,
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

        let tls_config = RustlsConfig::from_pem_file(
            PathBuf::from("cert").join("cert.crt"),
            PathBuf::from("cert").join("key.key"),
        )
        .await
        .expect("Missing certificate files");

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(self.db.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
        let test = self.clone();
        let app = Router::new()
            .nest("/admin", instance_handling::router::admin_router())
            .route_layer(permission_required!(Backend, Permissions::Admin))
            .merge(instance_handling::router::protected_router())
            .route_layer(login_required!(Backend))
            .merge(auth::router())
            .merge(new_user::router())
            .layer(auth_layer)
            .with_state(test);

        let port = var("API_PORT")?;
        axum_server::bind_rustls(
            std::net::SocketAddr::from_str(&format!("0.0.0.0:{port}")).unwrap(),
            tls_config,
        )
        .serve(app.into_make_service())
        .await
        .unwrap();

        Ok(())
    }
}
