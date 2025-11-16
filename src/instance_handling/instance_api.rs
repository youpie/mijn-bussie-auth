#![allow(dead_code)]

use std::str::FromStr;

use dotenvy::var;
use reqwest::{Response, StatusCode, Url};
use serde::Deserialize;
use strum::{AsRefStr, EnumString};

use crate::{
    GenResult,
    instance_handling::admin::passthrough::{InstanceGetRequests, InstancePostRequests},
};

#[allow(nonstandard_style)]
#[derive(AsRefStr, EnumString, Deserialize)]
pub enum KumaRequest {
    reset,
    delete,
}

pub struct Instance {}

impl Instance {
    async fn send_request(url: Url) -> reqwest::Result<Response> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?;
        client.get(url).send().await
    }

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

    fn create_base_kuma_url(user_name: Option<&str>, request: KumaRequest) -> GenResult<Url> {
        let mut url = Self::create_base_url(None)?.join("kuma/")?;
        url = url.join(request.as_ref())?;
        url = match user_name {
            Some(user_name) => url.join(user_name)?,
            None => url.join("all")?,
        };
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

    pub async fn refresh_user(user_name: Option<&str>) -> GenResult<bool> {
        let mut url = Self::create_base_url(None)?.join("refresh/")?;
        if let Some(user_name) = user_name {
            url = url.join(user_name)?;
        }

        url = Self::set_query(url);
        Ok(Self::verify_response(Self::send_request(url).await?))
    }

    pub async fn get_request(
        user_name: &str,
        request_type: InstanceGetRequests,
    ) -> GenResult<(StatusCode, String)> {
        let mut url = Self::create_base_url(Some(user_name))?.join(request_type.as_ref())?;
        url = Self::set_query(url);
        let request = Self::send_request(url).await?;
        Ok((request.status(), request.text().await?))
    }

    pub async fn post_request(
        user_name: &str,
        request_type: InstancePostRequests,
    ) -> GenResult<(StatusCode, String)> {
        let mut url = Self::create_base_url(Some(user_name))?.join(request_type.as_ref())?;
        url = Self::set_query(url);
        let request = Self::send_request(url).await?;
        Ok((request.status(), request.text().await?))
    }

    pub async fn kuma_request(
        user_name: Option<&str>,
        request: KumaRequest,
    ) -> GenResult<StatusCode> {
        let mut url = Self::create_base_kuma_url(user_name, request)?;
        url = Self::set_query(url);
        let request = Self::send_request(url).await?;
        Ok(request.status())
    }
}
