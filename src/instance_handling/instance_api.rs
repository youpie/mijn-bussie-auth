#![allow(dead_code)]

use std::str::FromStr;

use dotenvy::var;
use reqwest::{Response, StatusCode, Url};

use crate::GenResult;

pub struct Instance {}

impl Instance {
    fn create_base_url(user_name: Option<&str>) -> GenResult<Url> {
        let mut url = Url::from_str(&var("MIJN_BUSSIE_URL")?)?.join("api/")?;
        if let Some(user_name) = user_name {
            url = url.join(&format!("{user_name}/"))?;
        }
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

    fn set_query(mut url: Url) -> Url {
        url.set_query(Some(&format!(
            "key={}",
            var("API_KEY").expect("API key not set")
        )));
        url
    }

    pub async fn refresh_user(user_name: &str) -> GenResult<bool> {
        let mut url = Self::create_base_url(None)?
            .join("refresh/")?
            .join(user_name)?;
        url = Self::set_query(url);
        Ok(Self::verify_response(reqwest::get(url).await?))
    }

    pub async fn start_user(user_name: &str) -> GenResult<bool> {
        let mut url = Self::create_base_url(Some(user_name))?.join("start")?;
        url = Self::set_query(url);
        Ok(Self::verify_response(reqwest::get(url).await?))
    }

    pub async fn get_exit_code(user_name: &str) -> GenResult<(StatusCode, String)> {
        let mut url = Self::create_base_url(Some(user_name))?.join("ExitCode")?;
        url = Self::set_query(url);
        let request = reqwest::get(url).await?;
        Ok((request.status(), request.text().await?))
    }

    pub async fn get_calendar_link(user_name: &str) -> GenResult<(StatusCode, String)> {
        let mut url = Self::create_base_url(Some(user_name))?.join("Calendar")?;
        url = Self::set_query(url);
        let request = reqwest::get(url).await?;
        Ok((request.status(), request.text().await?))
    }

    pub async fn get_is_active(user_name: &str) -> GenResult<(StatusCode, String)> {
        let mut url = Self::create_base_url(Some(user_name))?.join("IsActive")?;
        url = Self::set_query(url);
        let request = reqwest::get(url).await?;
        Ok((request.status(), request.text().await?))
    }

    pub async fn get_logbook(user_name: &str) -> GenResult<(StatusCode, String)> {
        let mut url = Self::create_base_url(Some(user_name))?.join("Logbook")?;
        url = Self::set_query(url);
        println!("Sending admin request to {url:?}");
        let request = reqwest::get(url).await?;
        Ok((request.status(), request.text().await?))
    }
}
