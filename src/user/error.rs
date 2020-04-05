use diesel::result::Error as DieselError;
use failure::Fail;

pub type Result<T> = std::result::Result<T, UserError>;


#[derive(Fail, Debug, ToPrimitive)]
pub enum OperationError {
    #[fail(display = "找不到用户或登录方式")]
    NoSuchRecord = 404,
    #[fail(display = "凭据与用户不符")]
    CredentialNotValid = 401,
    #[fail(display = "未登录或权限不足")]
    Forbidden = 403,
    #[fail(display = "账户已禁用")]
    Disabled = 410,
    #[fail(display = "请求重复")]
    Conflict = 409,
    #[fail(display = "必须保证有一个登录方式")]
    NoMoreVerification = 418,
}

#[derive(Debug)]
pub enum UserError {
    // User operation error.
    OpError(OperationError),
    // Database Error.
    DBError(DieselError),
    // Parsing error.
    ParsingError,
    NetworkError,
    // WeChat error
    WechatError,
}

impl From<OperationError> for UserError {
    fn from(op_error: OperationError) -> UserError {
        UserError::OpError(op_error)
    }
}

impl From<DieselError> for UserError {
    fn from(db_error: DieselError) -> UserError {
        UserError::DBError(db_error)
    }
}


#[derive(Fail, Debug, ToPrimitive)]
pub(crate) enum WechatError {
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
