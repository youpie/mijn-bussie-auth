use std::fmt::Display;

use crate::GenResult;

pub trait OptionResult<T> {
    fn result(self) -> GenResult<T>;
    fn result_reason(self, reason: &str) -> GenResult<T>;
}

impl<T> OptionResult<T> for Option<T> {
    fn result(self) -> GenResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err("Option Unwrap".into()),
        }
    }
    fn result_reason(self, reason: &str) -> GenResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(reason.into()),
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
