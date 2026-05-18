# Logic & Patterns Review: mijn-bussie-auth

This covers the actual design decisions — enums, structs, data flow, and security. These are separate from the structural issues in the first review.

---

## 1. Two enums doing the same job, one of them hiding errors

You have two enums that represent the result of "does this instance already exist?":

```rust
// internal, in entity.rs
enum InstanceMatch {
    No,
    Username,
    Exact(UserDataModel),
}

// public, also in entity.rs
pub enum InstanceMatchReturn {
    NewUser(UserDataModel),
    Partial,
    Exact(UserDataModel),
    Unknown,   // ← this one
}
```

The mapping in `create_and_insert_instance`:
```rust
match self.find_existing_instance(db, &user_name).await {
    Some(InstanceMatch::Exact(m))  → InstanceMatchReturn::Exact(m)
    Some(InstanceMatch::No)        → InstanceMatchReturn::NewUser(...)
    Some(InstanceMatch::Username)  → InstanceMatchReturn::Partial
    None                           → InstanceMatchReturn::Unknown  // ← DB error!
}
```

`None` comes from `find_existing_instance` returning `None` only when the database query fails (`.all(db).await.ok()?` short-circuits on error). So `Unknown` does not mean "unknown state" — it means "the database failed". But the callers treat it like a normal variant:

```rust
// bypass/create_instance.rs
_ => (StatusCode::NOT_ACCEPTABLE).into_response()  // Unknown AND Partial both here
```

A database error returns `NOT_ACCEPTABLE` (422) to the client. That status code means "the request format is wrong." The real status should be 500, and the real fix is to not use an enum variant to signal errors. `find_existing_instance` should return `GenResult<InstanceMatch>` and the whole chain should use `?`. Then you do not need the `Unknown` variant at all, and errors are handled properly.

---

## 2. `MijnBussieInstance` is doing five different jobs

This struct is carrying the entire weight of the instance feature:

```rust
#[derive(Debug, DerivePartialModel, Deserialize, Serialize, Clone, Default)]
#[sea_orm(entity = "entity::user_data::Entity")]
pub struct MijnBussieInstance {
    // job 1: DB query projection (sea_orm attributes)
    pub user_data_id: i32,
    pub user_name: String,
    // job 2: JSON request body from client (serde)
    pub personeelsnummer: String,
    pub password: String,
    pub email: String,
    // job 3: skip-serializing rules for different contexts
    #[serde(skip_serializing_if = "Option::is_some", default)]
    pub name: Option<String>,
    #[serde(skip_deserializing, default)]
    pub online_created: bool,
    // job 4: nested relation (sea_orm nested)
    #[sea_orm(nested)]
    pub user_properties: user_properties::Model,
}

impl MijnBussieInstance {
    // job 5: all the business logic
    pub async fn create_and_insert_instance(...) { ... }
    pub async fn find_by_username(...) { ... }
    pub async fn get_all_users(...) { ... }
    pub fn _decrypt_values(...) { ... }
    pub fn calculate_execution_interval(...) { ... }
    pub async fn update_properties(...) { ... }
}
```

This is called the "God Struct" problem. Each job pulls the struct in a different direction: the DB projection wants all fields nullable for partial queries, the JSON request wants required fields, the JSON response wants some fields hidden, and the business logic wants to own the data transformations.

The sign that this is hurting you: the `#[serde(skip_serializing_if = "Option::is_some")]` on `name` is backwards — that attribute skips serialization when the `Option` *is* `Some`, which means it omits the name when it *has* a value. That's almost certainly not the intent.

The fix is separate types per job:

```rust
// What the client sends (POST body)
pub struct CreateInstanceRequest {
    pub personeelsnummer: String,
    pub password: String,
    pub email: String,
    pub properties: InstanceProperties,
}

// What comes back from the DB (read model)
pub struct InstanceRecord {
    pub user_data_id: i32,
    pub user_name: String,
    // ...all fields, never censored
}

// What gets sent to the client (response DTO)
pub struct InstanceResponse {
    pub user_name: String,
    pub email: String,      // decrypted
    // password deliberately absent
}
```

Each type has one job and its serde/sea_orm attributes are simple and correct.

---

## 3. `Instance` is an empty struct used as a module

```rust
pub struct Instance {}

impl Instance {
    async fn send_request(url: Url) -> reqwest::Result<Response> { ... }
    pub async fn get_request(...) { ... }
    pub async fn post_request(...) { ... }
    // etc.
}
```

`Instance` has no fields. All its methods are effectively free functions. In Rust, a zero-field struct with only static methods is just a module that makes you write `Instance::` instead of `instance_api::`. Delete the struct and turn the `impl` block into free functions in the `instance_api` module:

```rust
// instance_api.rs
pub async fn get_request(user_name: &str, kind: InstanceGetRequests) -> anyhow::Result<(StatusCode, String)> {
    let url = build_url(Some(user_name))?.join(kind.as_ref())?;
    let response = send(url).await?;
    Ok((response.status(), response.text().await?))
}
```

---

## 4. A new `reqwest::Client` is created for every HTTP call

```rust
async fn send_request(url: Url) -> reqwest::Result<Response> {
    let client = reqwest::Client::builder()   // ← new client every call
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()?;
    client.get(url).send().await
}
```

The reqwest documentation says explicitly: "The `Client` holds a connection pool internally, so it is advised that you create one and **reuse** it." Creating a new client per call bypasses connection pooling, adds TLS handshake overhead on every request, and wastes memory. A `reqwest::Client` is designed to be `Clone`d cheaply (it's `Arc`-backed), so put one in your `AppState` and reuse it:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub http: reqwest::Client,   // shared across all handlers
}

// In server startup:
let http = reqwest::Client::builder()
    .danger_accept_invalid_certs(true)
    .danger_accept_invalid_hostnames(true)
    .build()?;
```

---

## 5. Three pieces of dead code

**`user_allowed()` is never called.** Both `InstanceGetRequests` and `InstancePostRequests` have a `user_allowed()` method that checks whether a regular user can make that kind of request. The protection is actually done by axum-login middleware at the router level, so these methods are never consulted. Delete them.

```rust
impl InstanceGetRequests {
    fn user_allowed(&self) -> bool { ... }  // dead — remove
}
impl InstancePostRequests {
    fn user_allowed(&self) -> bool { ... }  // dead — remove
}
```

**`verify_response()` is never called.**

```rust
impl Instance {
    fn verify_response(response: Response) -> bool {  // dead — remove
        match response.status() { StatusCode::OK => true, _ => false }
    }
}
```

Also: `matches!(x, StatusCode::OK)` is the idiomatic form of this, not a `match` block.

**`_is_admin()` is prefixed with underscore, meaning it was never used after writing it.**

```rust
pub trait GetUser {
    async fn _is_admin(&self) -> bool;  // dead — remove
}
```

---

## 6. `censor()` has a copy-paste bug and a misleading name

```rust
impl Client for MijnBussieInstance {
    fn censor(self) -> Self {
        let mut empty_instance = MijnBussieInstance::default();
        let properties = &mut empty_instance.user_properties;

        // ...
        properties.send_mail_new_shift = properties_self.send_failed_signin_mail;  // ← WRONG FIELD
        properties.send_mail_removed_shift = properties_self.send_mail_removed_shift;
        properties.send_mail_updated_shift = properties_self.send_mail_updated_shift;
        // ...
    }
}
```

`send_mail_new_shift` is being assigned from `send_failed_signin_mail` — these are completely different settings. This is a copy-paste error.

The name `censor` is also misleading. "Censoring" implies stripping sensitive data from an existing record (like removing a password before sending to a client). But this method creates a *new* instance from scratch and copies selected fields, effectively constructing the view that the bypass endpoint will use for online-created accounts. A more accurate name would be `into_online_create_request()` or similar, making it obvious what it is actually building.

---

## 7. `Role` stored and compared as a raw `String`

In the database entity:

```rust
pub struct Model {
    pub role: String,   // "Admin" or "User" stored as text
    // ...
}
```

And in the backend:

```rust
async fn get_user_permissions(&self, user: &Self::User) -> BackendResult<HashSet<Self::Permission>> {
    hash_set.insert(Role::from_str(&user.inner.role).unwrap_or_default());
    // If role string is unrecognised, silently defaults to User
}
```

If someone inserts a role with a typo (`"admin"` instead of `"Admin"`, or `"ADMIN"`), `from_str` fails silently and the account gets `User` permissions — including someone who should be an admin. The `strum::EnumString` macro is case-sensitive by default.

The fix is to use `#[strum(ascii_case_insensitive)]` or to store the serialized form with `AsRefStr` consistently, and to propagate the parse error rather than swallowing it with `unwrap_or_default()`.

---

## 8. `backend_user` is a String foreign key to `user_name`

```rust
// user_account entity
pub backend_user: Option<String>,   // stores user_data.user_name
```

This links a user account to an instance by storing the instance's `user_name` as a plain string. This works, but it is a soft reference — if `user_name` ever changes, the link silently breaks. SeaORM and the DB (with a proper FK constraint on the `user_data.user_name` column) would enforce this. Looking at the entity definition, the relation is declared with `on_update = "Cascade"`, so the DB does cascade updates. But the risk is that application-level code can write `backend_user` values that do not exist in `user_data`, since that check only happens if the FK is enforced at the DB level.

More importantly: the account is linked to the instance by `user_name` (a human-readable username), not by `user_data_id` (a stable integer key). If you ever support renaming instances, this link needs updating separately.

---

## 9. `|` instead of `||` for boolean logic

```rust
pub fn calculate_execution_interval(&self) -> i32 {
    let properties = &self.user_properties;
    if properties.send_mail_new_shift
        | properties.send_mail_updated_shift   // ← bitwise OR
        | properties.send_mail_removed_shift
    { ... }
}
```

`|` is the bitwise OR operator. On booleans it gives the correct result (since `true | false == true`), but it does not short-circuit: all three values are always evaluated. For boolean logic in Rust you should use `||`, which short-circuits and is idiomatic:

```rust
if properties.send_mail_new_shift
    || properties.send_mail_updated_shift
    || properties.send_mail_removed_shift
{ ... }
```

---

## 10. `GetUser::get_user()` returns `StatusCode` as an error type

```rust
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
```

`StatusCode` is an HTTP concept. A function that extracts a user from a session should not know about HTTP — that knowledge belongs at the handler layer. This creates a coupling: if you ever want to call `get_user()` from non-HTTP code (tests, background tasks), you get HTTP status codes back as errors.

The better pattern is `Option<UserAccount>` or a domain error:

```rust
// Option — simple and honest
fn get_user(&self) -> Option<&UserAccount> {
    self.user.as_ref()
}

// Then in the handler, the HTTP concern lives where it belongs:
let user = auth_session.get_user().ok_or(AppError::Internal("no session user".into()))?;
```

---

## 11. Critical bug: the `personeelsnummer` conflict check always fires

In `new_instance.rs` (the protected "add instance" route):

```rust
if MijnBussieInstance::get_id_from_personeelsnummer(db, &instance.personeelsnummer)
    .await
    .ok()
    .is_some()
{
    return StatusCode::CONFLICT.into_response();
}
```

`get_id_from_personeelsnummer` returns `GenResult<Option<i32>>`, which is `Result<Option<i32>, Error>`.

Tracing the types:
- `.await` → `Result<Option<i32>, Error>`
- `.ok()` → converts to `Option<Option<i32>>`
- `.is_some()` → checks if the outer `Option` is `Some`

`Ok(None)` (no match found) becomes `Some(None)` after `.ok()`, which `.is_some()` considers true. This means **every** call to the protected add-instance route returns `409 CONFLICT`, even when the personeelsnummer does not exist. The correct form is:

```rust
if MijnBussieInstance::get_id_from_personeelsnummer(db, &instance.personeelsnummer)
    .await
    .ok()
    .flatten()   // ← this flattens Option<Option<i32>> to Option<i32>
    .is_some()
{
    return StatusCode::CONFLICT.into_response();
}
```

Or more clearly:

```rust
match get_id_from_personeelsnummer(db, &instance.personeelsnummer).await {
    Ok(Some(_)) => return StatusCode::CONFLICT.into_response(),
    Ok(None)    => { /* proceed */ }
    Err(e)      => return AppError::from(e).into_response(),
}
```

---

## 12. Security: the bypass password endpoint uses a calendar link as proof of identity

```rust
// bypass/change_password.rs
pub struct PasswordChange {
    pub calendar_link: String,      // ← used as identity proof
    pub personeelsnummer: String,
    pub password: String,
}

// The verification:
if calendar_link != new_password.calendar_link {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
}
```

The endpoint resets a backend password with no session or authentication — anyone who can call it can change anyone's password if they have the calendar link. Calendar links are subscription URLs designed to be shared with calendar applications and other people. They are not secrets. A better identity-proof mechanism for an unauthenticated endpoint would be a short-lived token sent to the user's registered email address.

---

## 13. `PasswordChange` is defined twice with different fields

```rust
// src/web/generic/account_management.rs
pub struct PasswordChange {
    pub password: String,
}

// src/bypass/change_password.rs
struct PasswordChange {
    pub calendar_link: String,
    pub personeelsnummer: String,
    pub password: String,
}
```

Two structs with the same name, in different modules, for different purposes. The bypass version should have a descriptive name like `BypassPasswordChange` or `UnauthenticatedPasswordReset` that makes the difference immediately clear.

---

## Summary

| Issue | Severity |
|---|---|
| `InstanceMatchReturn::Unknown` hides DB errors, maps to `NOT_ACCEPTABLE` | High |
| `MijnBussieInstance` God Struct (DB model + DTO + business logic) | High |
| `Instance` empty struct as fake namespace | Low |
| `reqwest::Client` created per-call (no connection pooling) | Medium |
| Dead code: `user_allowed()`, `verify_response()`, `_is_admin()` | Low |
| `censor()` copies wrong field (`send_failed_signin_mail → send_mail_new_shift`) | Critical |
| `censor()` misleading name | Low |
| `Role` stored as unvalidated `String`, parse errors silently default to `User` | Medium |
| `backend_user` is a soft string reference instead of an integer FK | Low |
| `|` instead of `||` for boolean conditions | Low |
| `GetUser::get_user()` returns HTTP `StatusCode` as domain error | Low |
| `Result<Option<T>>.ok().is_some()` always true — conflict check always fires | Critical |
| Calendar link used as identity proof for password reset | High (security) |
| Two structs named `PasswordChange` with different shapes | Low |
