use serde::Deserialize;

mod assign_instance;

#[derive(Deserialize)]
pub struct AdminQuery {
    pub user_name: String,
}
