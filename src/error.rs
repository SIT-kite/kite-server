use crate::models::wechat::WxErr;
use actix_http::error::PayloadError;
use actix_http::{http::StatusCode, ResponseBuilder};
use actix_web::{error::ResponseError, HttpResponse};
use failure::Fail;
use jsonwebtoken::errors::Error as JwtError;
use num_traits::ToPrimitive;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::Error as JsonError;
use sqlx::error::Error as SqlError;
use std::fmt;
use std::io::Error as StdIoError;

pub type Result<T> = std::result::Result<T, ApiError>;
pub type Error = ApiError;

// Reference:
// [Actix error handler](https://actix.rs/docs/errors/)
// [fmt::Display](https://doc.rust-lang.org/std/fmt/trait.Display.html)

/// Server error type, show internal library error with error code 1 and hide real error message.
/// While show logical and business errors with (code, message).
#[derive(Debug, Serialize, PartialEq)]
pub struct ApiError {
    pub code: u16,
    // TODO: Add inner error handler and the uncomment following line.
    #[serde(skip_serializing)]
    pub inner_msg: Option<String>,
    #[serde(rename(serialize = "msg"), skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerError {{code: {}, msg: {}}} ",
            self.code,
            self.error_msg.as_ref().unwrap_or(&String::from(""))
        )
    }
}

impl ResponseError for ApiError {
    // Always return 200 ok and prompt real code at json body.
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
    // Make json response body for error.
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).json(self)
    }
}

impl ApiError {
    pub fn new<T: ToPrimitive + Fail>(sub_err: T) -> Self {
        Self {
            code: sub_err.to_u16().unwrap(),
            inner_msg: None,
            error_msg: Some(sub_err.to_string()),
        }
    }
}

impl From<WxErr> for ApiError {
    fn from(e: WxErr) -> Self {
        ApiError {
            code: e.errcode,
            inner_msg: Some(e.errmsg),
            error_msg: None,
        }
    }
}

macro_rules! convert_inner_errors {
    ($src_err_type: ident) => {
        impl From<$src_err_type> for ApiError {
            fn from(sub_err: $src_err_type) -> Self {
                Self {
                    code: 1,
                    inner_msg: None,
                    error_msg: Some(sub_err.to_string()),
                }
            }
        }
    };
}

convert_inner_errors!(String);
convert_inner_errors!(PayloadError);
convert_inner_errors!(JsonError);
convert_inner_errors!(JwtError);
convert_inner_errors!(SqlError);
convert_inner_errors!(StdIoError);
