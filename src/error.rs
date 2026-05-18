use std::fmt::Display;

use anyhow::anyhow;
use axum::response::IntoResponse;
use hyper::StatusCode;
use thiserror::Error;

pub type GenResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub struct AppErrorContext {
    user_message: Option<&'static str>,
    admin_message: Option<&'static str>,
}

impl AppErrorContext {
    pub fn new_user(message: &'static str) -> Self {
        Self {
            user_message: Some(message),
            admin_message: None,
        }
    }
    pub fn user(&self) -> &'static str {
        match self.user_message {
            Some(message) => message,
            None => "",
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error occured")]
    Database(#[from] sea_orm::DbErr),
    #[error("User error: {:?}", ._0.admin_message)]
    UserError(AppErrorContext),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("User is not found")]
    NotFound,
    #[error("Multiple options")]
    Multiple(Vec<String>),
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::Generic(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
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

pub struct AnyErr(pub anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AnyErr {
    fn from(e: E) -> Self {
        AnyErr(e.into())
    }
}

impl From<AnyErr> for AppError {
    fn from(e: AnyErr) -> Self {
        AppError::Generic(e.0)
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
