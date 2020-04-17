use std::error::Error as StdError;
use std::fmt;

use actix_http::error::BlockingError;
use actix_http::error::PayloadError;
use actix_http::http::StatusCode;
use actix_http::ResponseBuilder;
use actix_web::{error::ResponseError, HttpResponse};
use deadpool_postgres::PoolError;
use failure::Fail;
use jsonwebtoken::errors::Error as JwtError;
use num_traits::cast::ToPrimitive;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::Error as JsonError;
use tokio_postgres::Error as PgError;

use crate::error::UserError;
use crate::user::wechat::WxErr;

// 自定义错误
// See: https://actix.rs/docs/errors/
// fmt::Display
// See: https://doc.rust-lang.org/std/fmt/trait.Display.html


// 对外部接口来说，逻辑错误返回相应的错误码，由各模块提供的错误类型提供错误码和提示信息
// 内部模块（如数据库、网络）错误返回错误类型 1（内部错误），并返回相应的错误信息提示
#[derive(Debug, Serialize)]
pub struct ServerError {
    #[serde(rename(serialize = "code"))]
    inner_code: u16,
    #[serde(rename(serialize = "msg"), skip_serializing_if = "String::is_empty")]
    error_msg: String,
}

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
            .body(serde_json::to_string(&self).unwrap_or(r"{code: 1}".to_string()))
    }
}

impl From<UserError> for ServerError {
    fn from(sub_error: UserError) -> Self {
        Self {
            inner_code: sub_error.to_u16().unwrap(),
            error_msg: sub_error.to_string(),
        }
    }
}



macro_rules! convert_inner_errors {
    ($src_err_type: ident) => {
    impl From<$src_err_type> for ServerError {
        fn from(sub_err: $src_err_type) -> Self {
            Self {
                inner_code: 1,
                error_msg: sub_err.to_string(),
            }
        }
    }}
}

convert_inner_errors!(String);
convert_inner_errors!(PayloadError);
convert_inner_errors!(WxErr);
convert_inner_errors!(JsonError);
convert_inner_errors!(PgError);
convert_inner_errors!(JwtError);