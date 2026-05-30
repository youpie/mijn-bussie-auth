# Code Review: mijn-bussie-auth

A structured review of the patterns and structural issues, with concrete fixes for each. The goal is not to rewrite — it is to show you the exact mistakes so you recognise them in future projects.

---

## 1. The single most painful problem: no typed HTTP error

Every single handler in the project repeats this shape:

```rust
// src/web/protected/account_handling.rs
match change_password(db, user.inner.username, new_password.password).await {
    Ok(_) => StatusCode::OK,
    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
}

// src/instance_handling/admin/instance_management.rs
match MijnBussieInstance::create_and_insert_instance(instance, db, true).await {
    Ok(InstanceMatchReturn::Exact(_)) => { ... }
    Ok(_) => StatusCode::OK.into_response(),
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
}
```

This is why adding new endpoints feels hard: you have to manually write the same `match`/`into_response()` glue every time. The solution is an `AppError` type that implements `IntoResponse` once, so your handlers just use `?` and are done.

```rust
// src/error.rs — replace what's there now

use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound(_)    => StatusCode::NOT_FOUND,
            AppError::BadRequest(_)  => StatusCode::BAD_REQUEST,
            AppError::Database(_)
            | AppError::Internal(_)  => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

// Convenience alias so handlers can use `?` directly
pub type AppResult<T> = Result<T, AppError>;
```

Now a handler becomes:

```rust
// Before
pub async fn change_password_protected(...) -> impl IntoResponse {
    let db = &data.db;
    let user = auth_session.user.expect("No user in protected space");
    match change_password(db, user.inner.username, new_password.password).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// After
pub async fn change_password_protected(...) -> AppResult<StatusCode> {
    let user = auth_session.user.ok_or(AppError::Internal("no session user".into()))?;
    change_password(&data.db, user.inner.username, new_password.password).await?;
    Ok(StatusCode::OK)
}
```

Every handler in the codebase can be simplified this way. The handler's return type becomes `AppResult<T>`, and internal functions can use the same type or return `GenResult<T>` and have a `From` impl do the conversion.

---

## 2. `GenResult` / `GenError` in `main.rs` should be `anyhow`

You have `anyhow` in `Cargo.toml` but you're not using it. Instead you've rolled your own box-error alias:

```rust
// main.rs
type GenResult<T> = Result<T, GenError>;
type GenError = Box<dyn std::error::Error + Send + Sync + 'static>;
```

This is exactly what `anyhow::Result` is, but `anyhow` is better: it captures stack traces, has `context()` for adding messages, and is more ergonomic than `"string".into()`. Replace all `GenResult<T>` with `anyhow::Result<T>` and all `Err("message".into())` with `anyhow::bail!("message")` or `Err(anyhow::anyhow!("message"))`.

```rust
// Instead of: Err("Empty credentials".into())
anyhow::bail!("empty credentials");

// Instead of: .result_reason("Could not deserialize password")?
.ok_or_else(|| anyhow::anyhow!("could not deserialize password"))?
```

You can then delete the entire `OptionResult` trait in `error.rs` — it only exists because `Box<dyn Error>` doesn't implement `From<()>`.

---

## 3. `main.rs` is a utility dumping ground

`main.rs` currently contains:
- Server startup (`main`)
- Encryption/decryption helpers (`encrypt_value`, `decrypt_value`)
- A domain trait (`Client`)

These do not belong in `main.rs`. `main.rs` should only start the program. Move things:

```
src/
  main.rs           ← only: install CryptoProvider, call Api::new().serve()
  crypto.rs         ← encrypt_value, decrypt_value (pub), secret loading
  traits.rs         ← Client trait (or put it in entity.rs near its impls)
  error.rs          ← AppError, AppResult
  web/
    ...
```

The cascade effect: `encrypt_value` is imported from `crate::` all over the codebase. If it lives in a dedicated module, it is easier to swap out, test, and reason about. Right now if you want to understand encryption you have to search `main.rs` and wonder if there is more.

---

## 4. Module structure mirrors roles in two places

You have two parallel hierarchies for "user role" (admin/protected/generic) that live in different top-level modules:

```
src/
  web/
    admin/           ← web-layer admin routes
    protected/       ← web-layer protected routes
    generic/         ← shared web logic
  instance_handling/
    admin/           ← instance admin routes
    protected/       ← instance protected routes
    generic/         ← shared instance logic
```

This means that to understand "what can an admin do?", you have to look in two completely separate places. And to add a new admin endpoint, you have to decide which tree it belongs to — and that decision is not obvious.

A cleaner structure organises by *feature* rather than by *framework layer*:

```
src/
  auth/             ← login, logout, signup, password
  account/          ← /me, /role, password change
  instance/
    mod.rs
    routes.rs       ← all instance routes, tagged by who can access them
    api.rs          ← the HTTP client that calls the backend
    entity.rs       ← MijnBussieInstance, DB queries
  admin/
    routes.rs       ← /admin/* routes: find, manage instances, assign
    query.rs        ← AdminQuery logic
  server.rs         ← Router assembly, TLS config, state
```

When you need to add a new instance endpoint, you open `instance/routes.rs`. When you add an admin endpoint, you open `admin/routes.rs`. You never have to guess.

---

## 5. The inline `mod get { }` / `mod post { }` pattern

Every file does this:

```rust
// src/web/auth.rs
pub fn router() -> Router<Api> { ... }

mod post {
    pub async fn login(...) { ... }
}

mod get {
    pub async fn logout(...) { ... }
}
```

Grouping by HTTP method is not useful because a file already only covers one feature. Grouping by method means each file has two or three nested modules with their own separate `use` blocks, which creates noise. A simpler pattern:

```rust
// src/auth/routes.rs — no nesting needed
pub fn router() -> Router<Api> {
    Router::new()
        .route("/login",  post(login))
        .route("/logout", get(logout))
}

pub async fn login(mut auth_session: AuthSession, Json(creds): Json<Credentials>) -> AppResult<StatusCode> {
    ...
}

pub async fn logout(mut auth_session: AuthSession) -> AppResult<StatusCode> {
    ...
}
```

Flat file, no nesting. The handler names (`login`, `logout`) already tell you the operation; the HTTP method is in the `router()` function. This is what large Axum projects do.

---

## 6. `AdminQuery` loads all database rows and filters in Rust

`AdminQuery` has four methods that all do the same thing: load every row from the database, then filter in Rust:

```rust
// src/instance_handling/admin/mod.rs
pub async fn get_instance_from_email(&self, db: &DatabaseConnection) -> Option<Vec<UserDataModel>> {
    if let Some(email) = &self.email {
        let users = user_data::Entity::find().all(db).await.ok(); // loads EVERYTHING
        if let Some(users) = users {
            let emails = users.into_iter()
                .filter(|user| {
                    decrypt_value(&user.email, true).ok().as_ref() == Some(&email.to_lowercase())
                })
                // ...
```

This is an N+1 / full-table-scan pattern. When you have 10 users it is invisible. With 1000 users it sends 1000 rows over the wire and decrypts them all on every admin API call.

The reason this happens is that the emails are encrypted in the DB, so you cannot filter by them in SQL. That is a legitimate constraint. But the correct response is to acknowledge that in a comment, not to silently scan everything:

```rust
// Note: emails are encrypted at rest, so SQL filtering is not possible.
// This performs a full table scan — acceptable for small user counts,
// but consider an indexed hash column if the table grows large.
pub async fn get_instance_from_email(&self, db: &DatabaseConnection) -> anyhow::Result<Vec<UserDataModel>> {
    let Some(email) = &self.email else { return Ok(vec![]) };
    let email_lower = email.to_lowercase();
    let all_users = user_data::Entity::find().all(db).await?;
    Ok(all_users.into_iter()
        .filter(|u| decrypt_value(&u.email, true).ok().as_deref() == Some(&email_lower))
        .collect())
}
```

Similarly, `get_account_from_instance` does a loop that fires one DB query per instance. Use `find_with_related` or an `IN` query instead.

Also in `find_existing_instance` inside `entity.rs`:

```rust
let existing_instances = user_data::Entity::find().all(db).await.ok()?; // loads all rows
let matching_instance = existing_instances.iter().find(|i| i.user_name == user_name);
```

This should be:

```rust
let matching_instance = user_data::Entity::find()
    .filter(user_data::Column::UserName.eq(user_name))
    .one(db)
    .await?;
```

---

## 7. `expect()` and `unwrap()` inside handlers

Protected handlers use `expect()` for the session user:

```rust
// src/web/protected/account_handling.rs
let user = auth_session.user.expect("No user in protected space");
```

The comment says "this can't happen in a protected space." But if the middleware ever changes, or there is a bug in axum-login, this panics the entire server process — not just that request. Panics in async Axum handlers do not kill one request; they can take down the whole tokio runtime. Use `?` with an error instead:

```rust
let user = auth_session.user.ok_or(AppError::Internal("no session user".into()))?;
```

Panicking `.unwrap()` calls in handlers (in `bypass/create_instance.rs`):

```rust
Instance::refresh_user(Some(&instance.user_name)).await.unwrap();
Instance::post_request(&instance.user_name, InstancePostRequests::Start).await.unwrap();
```

If the backend instance is down when a new user signs up via the bypass endpoint, this panics. Use `?` or handle the error explicitly and return a meaningful status code.

---

## 8. `Api` struct mixes state with startup

`Api` is used as Axum state (every handler gets `State(data): State<Api>`) but it also owns the server startup logic (`serve()`). These are two different concerns:

```rust
// Currently
pub struct Api {
    pub db: DatabaseConnection,
}
impl Api {
    pub async fn new() -> ... { ... }
    pub async fn serve(self) -> ... {
        // 50 lines of TLS, CORS, sessions, routes
    }
}
```

Rename to `AppState` (the idiomatic Axum name) and extract startup:

```rust
// src/server.rs
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

pub async fn run() -> anyhow::Result<()> {
    let state = AppState { db: connect_db().await? };
    let app = build_router(state);
    axum_server::bind_rustls(...).serve(app.into_make_service()).await?;
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/admin", admin::router())
        ...
        .with_state(state)
}
```

This way your state type is obviously just "things handlers need at runtime," and the server wiring is in one obvious place.

---

## 9. Replace `println!` with `tracing`

The codebase uses `println!` for all logging and has a custom `ResultLog` trait to print errors with a function name. The idiomatic Rust approach is the `tracing` crate, which gives you log levels, structured fields, and compatibility with log aggregators:

```rust
// Instead of:
println!("Connected to database!");
println!("Error in function \"{function_name}\": {}", err);
println!("Created new user with id {}", res.last_insert_id);

// Use:
tracing::info!("connected to database");
tracing::error!(function = function_name, error = %err, "operation failed");
tracing::info!(user_id = res.last_insert_id, "created new user");
```

Add `tracing` and `tracing-subscriber` to `Cargo.toml`, call `tracing_subscriber::fmt::init()` at the top of `main()`, and delete the custom `ResultLog` trait entirely.

---

## 10. Bug: `change_information` sets the wrong field for `personeelsnummer` and `user_name`

```rust
// src/instance_handling/generic/change_information.rs
if let Some(new_personeelsnummer) = &self.personeelsnummer {
    instance_data.email = Set(new_personeelsnummer.clone()); // ← sets EMAIL, not personeelsnummer
}
if let Some(new_username) = &self.user_name {
    instance_data.email = Set(new_username.clone()); // ← sets EMAIL again
}
```

Both branches set `instance_data.email` instead of `instance_data.personeelsnummer` and `instance_data.user_name` respectively. This is a silent data corruption bug — calling the change-instance-information endpoint with a `personeelsnummer` field will overwrite the user's email address.

---

## Summary table

| Problem | Severity | Fix |
|---|---|---|
| No typed HTTP error type | High — root cause of all boilerplate | Add `AppError: IntoResponse`, use `?` in handlers |
| `GenResult`/`GenError` in main.rs | Medium | Replace with `anyhow::Result` |
| Crypto + `Client` trait in main.rs | Medium | Move to `crypto.rs` / `traits.rs` |
| Role hierarchy duplicated across `web/` and `instance_handling/` | Medium | Reorganise by feature, not by role |
| Inline `mod get {}` / `mod post {}` pattern | Low | Flat files, named functions |
| `AdminQuery` full-table-scan filtering | Medium | SQL `WHERE` where possible, document where not |
| `find_existing_instance` full scan | Medium | Use `filter().one()` |
| `expect()` / `unwrap()` in handlers | High — can panic the server | Replace with `?` + `AppError` |
| `Api` mixes state with startup | Low | Rename to `AppState`, extract `run()` |
| `println!` and custom `ResultLog` | Low | Replace with `tracing` |
| Bug: `personeelsnummer`/`user_name` set wrong field | Critical | Fix the field assignments |
