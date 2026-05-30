use crate::instance_handling::{admin::AdminQuery, entity::MijnBussieInstance};

use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/names", get(get_name_list))
        .route("/emails", get(get_email_list))
        .route("/accounts", get(get_account_list))
}

async fn get_email_list(
    Query(user): Query<AdminQuery>,
    State(data): State<AppState>,
) -> GenResult<Json<Vec<String>>> {
    let db = &data.db;
    let all_users = get_users(db, user.to_option()).await;
    all_users
        .iter()
        .filter_map(|user| user.get_email().warn_owned("Decrypting email").ok())
        .collect::<Vec<String>>()
        .not_found()
        .map(|i| Json(i))
}

async fn get_name_list(
    Query(user): Query<AdminQuery>,
    State(data): State<AppState>,
) -> GenResult<Json<Vec<String>>> {
    let db = &data.db;
    let all_users = get_users(db, user.to_option()).await;

    all_users
        .iter()
        .filter_map(|user| user.get_name().ok())
        .collect::<Vec<String>>()
        .not_found()
        .map(|i| Json(i))
}

async fn get_account_list(
    State(data): State<AppState>,
    Query(users): Query<AdminQuery>,
) -> GenResult<Json<Vec<(String, String, Option<String>)>>> {
    let db = &data.db;
    let all_accounts = user_account::Entity::find()
        .find_with_related(user_data::Entity)
        .all(db)
        .await?;

    // If a user account has been specified. Print only that user
    let specific_user = users
        .get_user_account(db, true)
        .await
        .ok()
        .and_then(|account| Some(account.inner.username.clone()));
    let mut account_combination = vec![];
    for account in all_accounts {
        if specific_user.is_none() || Some(account.0.username.clone()) == specific_user {
            let linked_instance = account
                .1
                .first()
                .and_then(|instance| Some(instance.user_name.clone()));
            account_combination.push((account.0.username, account.0.role, linked_instance));
        }
    }

    account_combination.not_found().map(|i| Json(i))
}

async fn get_users(db: &DatabaseConnection, users: Option<AdminQuery>) -> Vec<MijnBussieInstance> {
    if let Some(user) = users {
        let instances = user.get_instance_name(db).await;
        let mut users = vec![];
        for instance in instances {
            match MijnBussieInstance::find_by_username(db, &instance).await {
                Ok(user) => users.push(user),
                Err(_) => continue,
            };
        }
        users
    } else {
        MijnBussieInstance::get_all_users(db)
            .await
            .warn_owned("Loading all users")
            .unwrap_or_default()
    }
}
