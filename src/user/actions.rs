use deadpool_postgres::Client;

use crate::error::{Result, ServerError, UserError};
use crate::user::models::{Person, Verification};

// Require to verify the credential and login.
// The function will return an error::Result. When the process success, an i32 value as uid
// will be returned. Otherwise, a UserError enum, provides the reason.
pub async fn login(client: &Client, username: String, password: String) -> Result<i32> {
    let statment = client.prepare(
        "SELECT uid FROM authentication WHERE login_type = 1 AND account = $1 AND credential = $2").await?;
    let rows = client.query(&statment, &[&username, &password]).await?;

    if !rows.is_empty() {
        return Ok(rows[0].try_get(0)?);
    }
    Err(UserError::LoginFailed)
}


pub async fn wechat_login(client: &Client, open_id: String) -> Result<i32> {
    let statment = client.prepare(
        "SELECT uid FROM authentication WHERE login_type = 0 AND account = $1").await?;
    let rows = client.query(&statment, &[&open_id]).await?;
    Ok(rows[0].try_get(0)?)
}

