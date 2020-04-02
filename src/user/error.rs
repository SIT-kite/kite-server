
use diesel::result::Error as DieselError;
use failure::Fail;
pub type Result<T> = std::result::Result<T, UserError>;

#[derive(Fail, Debug, ToPrimitive)]
pub enum OperationError {
    #[fail(display = "Could not find the user.")]
    NoSuchRecord = 404,
    #[fail(display = "Credential error, i.e. the password doesn't match the username.")]
    CredentialNotValid = 401,
    #[fail(display = "Forbidden for insufficient permissions.")]
    Forbidden = 403,
    #[fail(display = "The account is disabled.")]
    Disabled = 410,
    #[fail(display = "Repeated record.")]
    Conflict = 409,
    #[fail(display = "No more login approach.")]
    NoMoreVerification = 418,
}

#[derive(Debug)]
pub enum UserError {
    // User operation error.
    OpError(OperationError),
    // Database Error.
    DBError(DieselError),
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

