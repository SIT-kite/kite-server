use actix_http::error::PayloadError;
use failure::Fail;
use serde_json::Error as JsonError;
use tokio_postgres::Error as PgError;

pub use server::ServerError;
pub use user::UserError;

use crate::user::wechat::WxErr;

mod user;
mod server;

pub type Result<T> = std::result::Result<T, ServerError>;


// Document of failure library
// https://rust-lang-nursery.github.io/failure/derive-fail.html
#[derive(Fail, Debug)]
pub enum InnerError {
    #[fail(display = "Database unavailable: {}.", _0)]
    DbError(String),
    #[fail(display = "Parsing error.")]
    ParsingError(String),
    // todo.
    #[fail(display = "Network failure: {}.", _0)]
    NetworkError(String),
    #[fail(display = "External wechat interface error: {}", _0)]
    WechatError(WxErr),
}


impl From<PgError> for InnerError {
    fn from(pg_error: PgError) -> Self {
        InnerError::DbError(pg_error.to_string())
    }
}

impl From<JsonError> for InnerError {
    fn from(json_error: JsonError) -> Self {
        InnerError::ParsingError(json_error.to_string())
    }
}


impl From<WxErr> for InnerError {
    fn from(wx_interface_err: WxErr) -> Self {
        InnerError::WechatError(wx_interface_err)
    }
}

impl From<PayloadError> for InnerError {
    fn from(payload_error: PayloadError) -> Self {
        InnerError::NetworkError(payload_error.to_string())
    }
}