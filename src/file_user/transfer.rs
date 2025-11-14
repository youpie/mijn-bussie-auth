use std::path::PathBuf;

use sea_orm::DatabaseConnection;

use crate::{GenResult, add_new_user_to_db, file_user::file::load_user};

pub async fn tranfer_user_from_path(db: &DatabaseConnection, path: &PathBuf) -> GenResult<i32> {
    if !path.is_relative() {
        return Err("Path should be relative!".into());
    }
    let data = load_user(path)?;
    let user = add_new_user_to_db(&db, data.0, data.1).await?;
    Ok(user.user_data_id)
}
