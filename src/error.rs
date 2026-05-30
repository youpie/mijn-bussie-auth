use std::fmt::Display;

use anyhow::anyhow;
use axum::response::IntoResponse;
use hyper::StatusCode;
use thiserror::Error;
use tracing::warn;

use crate::prelude;

pub type GenResult<T> = Result<T, AppError>;

#[derive(Debug, PartialEq)]
pub struct AppErrorContext {
    user_message: Option<String>,
    admin_message: Option<String>,
}

impl AppErrorContext {
    pub fn new_user(message: String) -> Self {
        Self {
            user_message: Some(message),
            admin_message: None,
        }
    }
    pub fn user(&self) -> String {
        match &self.user_message {
            Some(message) => message.to_owned(),
            None => "".to_owned(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error occured: {:?}", _0.to_string())]
    Database(#[from] sea_orm::DbErr),
    #[error(transparent)]
    InstanceError(#[from] crate::instance_handling::instance_api::InstanceApiError),
    #[error("User error: {:?}", ._0.admin_message)]
    UserError(AppErrorContext),
    #[error("Parse error: {}", 0.to_string())]
    Parse(#[from] serde_json::Error),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("User is not found")]
    NotFound,
    #[error("User already exists")]
    AlreadyExists,
    #[error("Multiple options")]
    Multiple(Vec<String>),
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        warn!("{}", &self.to_string());
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::Generic(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::InstanceError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            Self::AlreadyExists => StatusCode::CONFLICT.into_response(),
            Self::Parse(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::UserError(error) => (StatusCode::BAD_REQUEST, error.user()).into_response(),
            Self::Multiple(options) => (
                StatusCode::MULTIPLE_CHOICES,
                serde_json::to_string_pretty(&options).unwrap_or_default(),
            )
                .into_response(),
        }
    }
}

pub trait IntoAnyhow<T> {
    fn d(self) -> Result<T, anyhow::Error>;
}

impl<T, E: Into<anyhow::Error>> IntoAnyhow<T> for Result<T, E> {
    fn d(self) -> Result<T, anyhow::Error> {
        self.map_err(Into::into)
    }
}

pub trait NotFound<T> {
    fn not_found(self) -> GenResult<T>;
}

impl<T> NotFound<T> for Option<T> {
    /// Map an option `None` to a `AppError::NotFound`
    fn not_found(self) -> GenResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(AppError::NotFound),
        }
    }
}

impl<T> NotFound<Vec<T>> for Vec<T> {
    /// Map an empty `vec` to a `AppError::NotFound`
    fn not_found(self) -> GenResult<Vec<T>> {
        if self.is_empty() {
            Err(AppError::NotFound)
        } else {
            Ok(self)
        }
    }
}

pub trait OptionResult<T> {
    fn result(self) -> GenResult<T>;
    fn result_reason(self, reason: &str) -> GenResult<T>;
}

impl<T> OptionResult<T> for Option<T> {
    fn result(self) -> GenResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(AppError::NotFound),
        }
    }
    fn result_reason(self, reason: &str) -> GenResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(anyhow!("{}", reason).into()),
        }
    }
}

pub trait ResultLog<T, E> {
    fn error(&self, function_name: &str);
    fn warn(&self, function_name: &str);
    fn warn_owned(self, function_name: &str) -> Self;
    fn info(&self, function_name: &str);
}

impl<T, E> ResultLog<T, E> for Result<T, E>
where
    E: Display,
{
    fn info(&self, function_name: &str) {
        match self {
            Err(err) => {
                println!("Error in function \"{function_name}\": {}", err.to_string())
            }
            _ => (),
        }
    }
    fn warn_owned(self, function_name: &str) -> Self {
        self.inspect_err(|err| {
            println!("Error in function \"{function_name}\": {}", err.to_string())
        })
    }
    fn warn(&self, function_name: &str) {
        match self {
            Err(err) => {
                println!("Error in function \"{function_name}\": {}", err.to_string())
            }
            _ => (),
        }
    }
    fn error(&self, function_name: &str) {
        match self {
            Err(err) => {
                println!("Error in function \"{function_name}\": {}", err.to_string())
            }
            _ => (),
        }
    }
}

pub trait ToString {
    fn to_string(&self) -> String;
}

impl<T: std::fmt::Debug> ToString for GenResult<T> {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
