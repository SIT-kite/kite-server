use deadpool_postgres::{Client, ClientWrapper};
use serde::{Deserialize, Serialize};

use crate::error::{Result, ServerError, UserError};
use crate::user::models::{LOGIN_USERNAME, LOGIN_WECHAT, Person, Verification};

// Require to verify the credential and login.
// The function will return an error::Result. When the process success, an i32 value as uid
// will be returned. Otherwise, a UserError enum, provides the reason.
pub async fn login(client: &Client, username: String, password: String) -> Result<i32> {
    let statement = client.prepare(
        "SELECT uid FROM authentication WHERE login_type = 1 AND account = $1 AND credential = $2 LIMIT 1").await?;
    let row = client.query_opt(&statement, &[&username, &password]).await?;

    match row {
        Some(record) => Ok(record.get(0)),
        None => Err(ServerError::from(UserError::LoginFailed)),
    }
}


pub async fn wechat_login(client: &Client, open_id: String) -> Result<i32> {
    let statement = client.prepare(
        "SELECT uid FROM authentication WHERE login_type = 0 AND account = $1 LIMIT 1").await?;
    let row = client.query_opt(&statement, &[&open_id]).await?;

    match row {
        Some(record) => Ok(record.get(0)),
        None => Err(ServerError::from(UserError::LoginFailed)),
    }
}


pub async fn get_authentication_count(client: &Client, uid: i32, login_type: i32) -> Result<i64> {
    // 注意这里的 client.query_one()
    // 官方文档 https://docs.rs/tokio-postgres/0.5.3/tokio_postgres/struct.Client.html#method.query_one
    // 指出，如果记录条数不为1, 则返回错误。下面查询能保证始终返回一行
    let statement = client.prepare(
        "SELECT COUNT(uid) FROM authentication WHERE uid = $1 AND login_type = $2").await?;
    let row = client.query_one(&statement, &[&uid, &login_type]).await?;
    // PostgreSQL 内部使用 8 位整数表示行号
    Ok(row.get::<_, i64>(0))
}


pub async fn bind_password(client: &Client, uid: i32, username: String, password: String) -> Result<()> {
    // 目前不允许一个账号绑定多个用户名密码或者微信账户
    if get_authentication_count(client, uid, LOGIN_USERNAME).await? != 0 {
        return Err(ServerError::from(UserError::AuthExists));
    }
    let statement = client.prepare(
        "INSERT INTO authentication (uid, login_type, account, credential) VALUES ($1, $2, $3, $4)").await?;
    let _ = client.execute(&statement, &[&uid, &LOGIN_USERNAME, &username, &password]).await?;

    Ok(())
}


pub async fn bind_wechat(client: &Client, uid: i32, openid: String) -> Result<()> {
    // 目前不允许一个账号绑定多个用户名密码或者微信账户
    if get_authentication_count(client, uid, LOGIN_USERNAME).await? != 0 {
        return Err(ServerError::from(UserError::AuthExists));
    }
    let statement = client.prepare(
        "INSERT INTO authentication (uid, login_type, account) VALUES ($1, $2, $3)").await?;
    let _ = client.execute(&statement, &[&uid, &LOGIN_WECHAT, &openid]).await?;

    Ok(())
}


#[derive(Serialize, Deserialize)]
pub struct UserExtra {
    pub gender: Option<i32>,
    #[serde(rename = "nickName", skip_serializing)]
    pub nick_name: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub language: Option<String>,
}

pub struct User {
    uid: i32,
    nick_name: Option<String>,
    extra: Option<UserExtra>,

    role: i16,
    is_disabled: bool,
}


pub async fn create_user(client: &Client, nick_name: String, extra: UserExtra) -> Result<i32> {
    // let extra  = serde_json::to_string(&extra).unwrap();
    let extra = serde_json::to_value(extra)?;
    let statement = client.prepare(
        "INSERT INTO persons (nick_name, extra) VALUES ($1, $2) RETURNING (uid)").await?;
    let rows = client.query(&statement, &[&nick_name, &extra]).await?;

    if rows.len() != 0 {
        return Ok(rows[0].try_get(0)?);
    }
    Err(ServerError::from(UserError::Unknown))
}


pub fn get_default_avatar() -> &'static str {
    "https://"
}


pub async fn list_users(client: &Client, page_index: i32, page_size: i32) -> Result<Vec<User>> {
    let statement = client.prepare(
        "SELECT (uid, nick_name, is_disabled, role, extra) FROM persons LIMIT $1 OFFSET $2").await?;
    let rows = client.query(&statement, &[&page_size, &page_index]).await?;
    let mut users = Vec::new();

    for row in rows {
        let extra = if row.try_get::<_, &str>(4).is_ok() {
            Some(serde_json::from_str(row.get::<_, &str>(4))?)
        } else {
            None
        };
        users.push(User {
            uid: row.get(0),
            nick_name: if row.try_get::<_, &str>(1).is_ok() { Some(row.get::<_, &str>(1).to_string()) } else { None },
            is_disabled: row.get(2),
            role: row.get(3),
            extra,
        });
    }
    Ok(users)
}


pub async fn query_user(client: &Client, uid: i32) -> Result<User> {
    // 因为 uid 为 unique, 下列语句最多返回一行
    let statement = client.prepare(
        "SELECT (nick_name, role, extra) FROM persons WHERE uid = $1").await?;
    let user = client.query_opt(&statement, &[&uid]).await?;

    match user {
        Some(row) => {
            let extra = if row.try_get::<_, &str>(4).is_ok() {
                Some(serde_json::from_str(row.get::<_, &str>(4))?)
            } else {
                None
            };
            Ok(User {
                uid,
                nick_name: if row.try_get::<_, &str>(0).is_ok() { Some(row.get::<_, &str>(1).to_string()) } else { None },
                role: row.get(1),
                extra,
                is_disabled: false,
            })
        },
        None => Err(ServerError::from(UserError::NoSuchUser))
    }
}

