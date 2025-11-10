#![allow(dead_code)]

use std::str::FromStr;

use dotenvy::var;
use reqwest::{Response, StatusCode, Url};

use crate::{
    GenResult,
    web::user::{AuthSession, GetUser},
};

pub struct Instance {}

impl Instance {
    fn create_base_url() -> GenResult<Url> {
        let mut url = Url::from_str(&var("MIJN_BUSSIE_URL")?)?.join("api/")?;
        url.set_query(Some(&format!(
            "key={}",
            var("API_KEY").expect("API key not set")
        )));
        Ok(url)
    }

    fn verify_response(response: Response) -> bool {
        match response.status() {
            StatusCode::OK => true,
            _ => false,
        }
    }

    pub async fn refresh_user(user_name: &str) -> GenResult<bool> {
        let url = Self::create_base_url()?.join("refresh/")?.join(user_name)?;
        Ok(Self::verify_response(reqwest::get(url).await?))
    }

    pub async fn start_user(user_name: &str) -> GenResult<bool> {
        let url = Self::create_base_url()?.join(user_name)?.join("/start")?;
        Ok(Self::verify_response(reqwest::get(url).await?))
    }

    pub async fn get_exit_code(user_name: &str) -> GenResult<String> {
        let url = Self::create_base_url()?
            .join(user_name)?
            .join("/ExitCode")?;
        Ok(String::from_utf8(
            reqwest::get(url).await?.bytes().await?.to_vec(),
        )?)
    }

    pub async fn get_logbook(user_name: &str, auth_session: &AuthSession) -> GenResult<String> {
        if !auth_session.is_admin().await {
            return Err("Admin required".into());
        }
        let url = Self::create_base_url()?.join(user_name)?.join("/Logbook")?;
        Ok(String::from_utf8(
            reqwest::get(url).await?.bytes().await?.to_vec(),
        )?)
    }
}
