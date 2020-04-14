use std::error::Error as StdError;
use std::fmt;

use actix_http::error::BlockingError;
use actix_http::http::StatusCode;
use actix_http::ResponseBuilder;
use actix_web::{error::ResponseError, HttpResponse};
use deadpool_postgres::PoolError;
use failure::_core::fmt::Error;
use failure::Fail;
use num_traits::cast::ToPrimitive;
use serde::export::Formatter;
use serde::Serialize;

use crate::user::error::UserError;

// Setting custom error
// See: https://actix.rs/docs/errors/
// fmt::Display
// See: https://doc.rust-lang.org/std/fmt/trait.Display.html


#[derive(Debug, Serialize)]
pub struct ServerError {
    inner_code: u16,
    error_msg: String,
}

// macro_rules

macro_rules! convert_custom_error {
    ($sub_error_type: ident) => {
    impl From<$sub_error_type> for ServerError {
        fn from(sub_error: $sub_error_type) -> Self
        {
            Self {
                inner_code: sub_error.code(),
                error_msg: sub_error.to_string(),
            }
        }
    }}
}


macro_rules! convert_standard_error {
    ($sub_error_type: ident) => {
    impl From<$sub_error_type> for ServerError {
        fn from(sub_error: $sub_error_type) -> Self
        {
            Self {
                inner_code: 1,
                error_msg: sub_error.to_string(),
            }
        }
    }}
}

convert_custom_error!(UserError);
convert_standard_error!(PoolError);


impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ServerError {{code: {}, msg: {}}} ", self.inner_code, self.error_msg)
    }
}
impl ResponseError for ServerError {
    // Always return 200 ok and prompt real code at json body.
    fn status_code(&self) -> StatusCode { StatusCode::OK }
    // Make json response body for error.
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .body(serde_json::to_string(&self).unwrap())
    }
}

