use chrono::{DateTime, Local};
use sqlx::PgPool;
use validator::{Validate, ValidationError};

use crate::error::{ApiError, Result};
use crate::portal::{Credential, Portal};

/// 完整的用户信息结构
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// 用户 ID
    pub uid: i32,
    /// 用户学号
    pub account: String,
    /// 用户创建日期
    pub create_time: DateTime<Local>,
    /// 账户是否被禁用
    pub is_block: bool,
}

pub enum UserError {
    NotFound,
    AlreadyExists,
    FormatInvalid,
}

pub struct Validator;

impl Validator {
    pub fn validate_username(username: &str) -> bool {
        // TODO: 参考 kite-app 中对学号的验证，补全用户名（学号）校验逻辑。
        return true;
    }
}

pub async fn login(pool: &PgPool, credential: &Credential) -> Result<User> {
    // 在数据库中查询信息
    let user =
        sqlx::query_as("SELECT uid, account, create_time, is_block FROM user.account WHERE username = $1 LIMIT 1;")
            .bind(&credential.account)
            .fetch_optional(pool)
            .await?;
    user.ok_or(ApiError::new(UserError::NotFound))
}

pub async fn create(account: &str) -> Result<i32> {
    let user: (i32,) = sqlx::query_as("INSERT INTO user.account (account) VALUES($1) RETURNING (uid);")
        .bind(account)
        .fetch_one(pool)
        .await?;
    Ok(user.0)
}
