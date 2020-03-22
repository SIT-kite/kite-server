
use diesel::result::Error as DieselError;


pub type Result<T> = std::result::Result<T, UserError>;

pub enum OperationError {
    // Could not find the user.
    UserNotFound,
    // Credential error, i.e. the password doesn't match the username. 401
    CredentialNotValid,
    // Forbidden, usually when some normal users are going to access
    // Admin page. 403
    Forbidden,
    // The account is disabled, user should ask the administrator for more detail. 410
    Disabled,
    // Repeated record. Use HTTP 409: Conflict.
    Conflict,
    // Warning: No more login approach. Use HTTP 418
    NoMoreVerification,
    // Could not find the verification way.
    NoSuchVerification,
}

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
