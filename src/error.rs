use std::io::Error as IoError;
use jsonwebtoken::errors::Error as JwtError;
use num_traits::ToPrimitive;
use poem::error::ResponseError;
use poem::http::StatusCode;
use reqwest::Error as ReqwestError;
use serde::de::StdError;
use serde_json::Error as JsonError;
use sqlx::error::Error as SqlxError;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub struct ApiError {
    pub code: u16,
    pub msg: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl ResponseError for ApiError {
    fn status(&self) -> StatusCode {
        StatusCode::OK
    }

    /// Convert this error to a HTTP response.
    fn as_response(&self) -> poem::Response
    where
        Self: StdError + Send + Sync + 'static,
    {
        poem::Response::builder()
            .status(self.status())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&self).unwrap())
    }
}

impl ApiError {
    pub fn new<T: ToPrimitive + std::error::Error>(sub_err: T) -> Self {
        Self {
            code: sub_err.to_u16().unwrap(),
            msg: Some(sub_err.to_string()),
        }
    }

    pub fn custom(code: u16, msg: &str) -> Self {
        Self {
            code,
            msg: Some(msg.to_string()),
        }
    }
}

#[macro_export]
macro_rules! convert_inner_errors {
    ($src_err_type: ident) => {
        impl From<$src_err_type> for ApiError {
            fn from(sub_err: $src_err_type) -> Self {
                Self {
                    code: 1,
                    msg: Some(sub_err.to_string()),
                }
            }
        }
    };
}

convert_inner_errors!(SqlxError);
convert_inner_errors!(JsonError);
convert_inner_errors!(ReqwestError);
convert_inner_errors!(JwtError);
convert_inner_errors!(IoError);