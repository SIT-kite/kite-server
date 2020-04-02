
use std::fmt;
use actix_http::ResponseBuilder;
use actix_web::{error::ResponseError, HttpResponse};
use crate::user::error::*;
use actix_http::http::StatusCode;
use serde::export::Formatter;
use num_traits::cast::ToPrimitive;
use diesel::r2d2::PoolError;
use actix_http::error::BlockingError;

// Setting custom error
// See: https://actix.rs/docs/errors/
// fmt::Display
// See: https://doc.rust-lang.org/std/fmt/trait.Display.html


#[derive(Debug)]
pub enum ServerError{
    User(UserError),
    Pool(PoolError),
    Block,
    // more
}



impl From<UserError> for ServerError {
    fn from(user_error: UserError) -> Self {
        ServerError::User(user_error)
    }
}

impl From<PoolError> for ServerError {
    fn from(pool_error: PoolError) -> Self {
        ServerError::Pool(pool_error)
    }
}


impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::User(user_error) => match user_error {
                UserError::OpError(e) => {
                    StatusCode::from_u16(e.to_u16().unwrap()).unwrap()
                },
                UserError::DBError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ServerError::Pool(pool_error) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            ServerError::Block => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).body("")
    }
}


impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.status_code())
    }
}
