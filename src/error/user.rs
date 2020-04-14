use failure::Fail;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::error::ServerError;

#[derive(Fail, Debug, ToPrimitive)]
pub enum UserError {
    #[fail(display = "Insufficient permission")]
    Forbidden = 4,
    #[fail(display = "Invalid credentials")]
    LoginFailed = 5,
}



