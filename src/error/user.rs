use failure::Fail;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::error::ServerError;

#[derive(Fail, Debug, ToPrimitive)]
pub enum UserError {
    #[fail(display = "参数错误")]
    BadParameter = 2,
    #[fail(display = "权限不足")]
    Forbidden = 4,
    #[fail(display = "凭据无效")]
    LoginFailed = 5,
    #[fail(display = "未知错误")]
    Unknown = 7,
    #[fail(display = "存在冲突的登录凭据")]
    AuthExists = 15,
    #[fail(display = "找不到用户")]
    NoSuchUser = 16,
}



