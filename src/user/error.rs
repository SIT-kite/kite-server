use failure::Fail;
use num_traits::ToPrimitive;

pub type Result<T> = std::result::Result<T, UserError>;
type OpErrType = OperationErrorType;
type WxErrType = WechatErrorType;

#[derive(Fail, Debug, ToPrimitive)]
pub enum OperationErrorType {
    #[fail(display = "Insufficient permission")]
    Forbidden,
    #[fail(display = "Invalid credentials")]
    LoginFailed,
}

// Document of failure library
// https://rust-lang-nursery.github.io/failure/derive-fail.html
#[derive(Fail, Debug)]
#[fail(display = "UserModule")]
pub enum UserError {
    #[fail(display = "Normal: {}.", _0)]
    OpError(OpErrType),
    #[fail(display = "Database.")]
    DbError,
    #[fail(display = "Parsing.")]
    ParsingError,
    #[fail(display = "Network.")]
    NetworkError,
    #[fail(display = "Wechat.")]
    WechatError,
}

#[derive(Fail, Debug, ToPrimitive)]
pub enum WechatErrorType {
    #[fail(display = "系统繁忙")]
    Busy = -1,
    #[fail(display = "无错误")]
    Ok = 0,
    #[fail(display = "AppSecret 不符")]
    AppSecret = 40001,
    #[fail(display = "grant_type 字段不为 client_credential")]
    GrantType = 40002,
    #[fail(display = "AppID 错误")]
    AppId = 40013,
    #[fail(display = "code 无效")]
    CodeErr = 40029,
    #[fail(display = "请求过于频繁")]
    TooFrequent = 45011,
}


impl UserError {
    pub fn code(&self) -> u16 {
        match self {
            UserError::OpError(e) => e.to_u16().unwrap_or(1),
            // Internal server error
            // TODO: Record internal error message for debug.
            _ => 1,
        }
    }
}

impl From<OpErrType> for UserError {
    fn from(op_error: OpErrType) -> UserError {
        UserError::OpError(op_error)
    }
}


