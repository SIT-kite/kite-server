use crate::user::error::Result;
use crate::user::models::{Person, Verification};

// Require to verify the credential and login.
// The function will return an error::Result. When the process success, an i32 value as uid
// will be returned. Otherwise, a UserError enum, provides the reason.
//pub fn login(conn: &PgConnection, auth: Verification) -> Result<i32> {
//
//    // Return no such user found.
//    Err(OpError(NoSuchRecord))
//}
