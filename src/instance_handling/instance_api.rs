#![allow(dead_code)]

use std::str::FromStr;

use dotenvy::var;
use reqwest::{Client, Response, StatusCode, Url};
use serde::Deserialize;
use strum::{AsRefStr, EnumString};
use thiserror::Error;

use super::*;

#[derive(Debug, Error)]
pub enum InstanceApiError {
    #[error("Reqwest error: {}",0.to_string())]
    Reqwest(#[from] reqwest::Error),
    #[error("URL parse error: {}",0.to_string())]
    Url(#[from] url::ParseError),
    #[error("ENV parse error: {}",0.to_string())]
    Env(#[from] dotenvy::Error),
    #[error("Instance Not Found")]
    NotFound,
    #[error("Instance Backend is Offline: {0}")]
    Offline(String),
    #[error("API Key Incorrect")]
    IncorrectKey,
    #[error("Instance had an Error: {0}")]
    InstanceError(String),
}

trait MapResponse {
    /// Map the Reqwest response to what that means for the Instance
    async fn map_response(self) -> Result<Response, InstanceApiError>;
}

impl MapResponse for Result<Response, reqwest::Error> {
    async fn map_response(self) -> Result<Response, InstanceApiError> {
        match self {
            Ok(response) => match response.status() {
                StatusCode::NO_CONTENT => Err(InstanceApiError::NotFound),
                StatusCode::INTERNAL_SERVER_ERROR => Err(InstanceApiError::InstanceError(
                    response.text().await.unwrap_or_default(),
                )),
                StatusCode::UNAUTHORIZED => Err(InstanceApiError::IncorrectKey),
                _ => Ok(response),
            },
            Err(e) if e.is_connect() => Err(InstanceApiError::Offline(e.to_string())),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(AsRefStr, EnumString, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum KumaRequest {
    Reset,
    Delete,
}

#[derive(Debug, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InstanceGetRequests {
    Logbook,
    IsActive,
    ExitCode,
    Name,
    Calendar,
    Standing,
    Logs,
}

#[derive(Debug, Deserialize, AsRefStr, PartialEq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InstancePostRequests {
    Start,
    Delete,
    Welcome,
}

pub fn create_client() -> Result<Client, InstanceApiError> {
    Ok(reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()?)
}

fn create_base_url(user_name: Option<&str>) -> Result<Url, InstanceApiError> {
    let mut url = Url::from_str(&var("MIJN_BUSSIE_URL")?)?.join("api/")?;
    if let Some(user_name) = user_name {
        url = url.join(&format!("{user_name}/"))?;
    }
    url.set_query(Some(&format!(
        "key={}",
        var("API_KEY").map_err(|_err| InstanceApiError::IncorrectKey)?
    )));
    Ok(url)
}

fn create_base_kuma_url(
    user_name: Option<&str>,
    request: KumaRequest,
) -> Result<Url, InstanceApiError> {
    let mut url = create_base_url(None)?.join("kuma/")?;
    url = url.join(&format!("{}/", request.as_ref().to_ascii_lowercase()))?;
    url = match user_name {
        Some(user_name) => url.join(user_name)?,
        None => url.join("all")?,
    };
    Ok(url)
}

fn set_query(mut url: Url) -> Url {
    url.set_query(Some(&format!(
        "key={}",
        var("API_KEY").expect("API key not set")
    )));
    url
}

async fn send_request(client: &Client, url: Url) -> Result<Response, InstanceApiError> {
    Ok(client.get(url).send().await.map_response().await?)
}

pub async fn refresh_user(
    client: &Client,
    user_name: Option<&str>,
) -> Result<String, InstanceApiError> {
    let mut url = create_base_url(None)?;
    if let Some(user_name) = user_name {
        url = url.join(&format!("refresh/{user_name}"))?;
    } else {
        url = url.join("refresh")?;
    }

    url = set_query(url);
    let request = send_request(client, url).await?;
    Ok(request.text().await?)
}

pub async fn get_request(
    client: &Client,
    user_name: &str,
    request_type: InstanceGetRequests,
) -> Result<String, InstanceApiError> {
    let mut url = create_base_url(Some(user_name))?.join(request_type.as_ref())?;
    url = set_query(url);
    let request = send_request(client, url).await?;
    Ok(request.text().await?)
}

pub async fn post_request(
    client: &Client,
    user_name: &str,
    request_type: InstancePostRequests,
) -> Result<String, InstanceApiError> {
    let mut url = create_base_url(Some(user_name))?.join(request_type.as_ref())?;
    url = set_query(url);
    let request = send_request(client, url).await?;
    Ok(request.text().await?)
}

pub async fn kuma_request(
    client: &Client,
    user_name: Option<&str>,
    request: KumaRequest,
) -> Result<(), InstanceApiError> {
    let mut url = create_base_kuma_url(user_name, request)?;
    url = set_query(url);
    send_request(client, url).await?;
    Ok(())
}

impl InstanceGetRequests {
    fn user_allowed(&self) -> bool {
        matches!(
            self,
            InstanceGetRequests::Calendar
                | InstanceGetRequests::Name
                | InstanceGetRequests::ExitCode
        )
    }
}

impl InstancePostRequests {
    fn user_allowed(&self) -> bool {
        matches!(self, InstancePostRequests::Delete)
    }
}
