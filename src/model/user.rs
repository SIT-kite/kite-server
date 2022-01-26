use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::error::Result;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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
    /// 用户角色
    pub role: i32,
    /// 账户是否被禁用
    pub is_block: bool,
}

#[derive(Debug, num_derive::ToPrimitive, thiserror::Error)]
pub enum UserError {
    #[error("用户账户格式不正确")]
    InvalidAccountFormat = 2,
    #[error("缺少登录凭据")]
    CredentialMissing = 3,
}

pub struct Validator;

impl Validator {
    pub fn validate_username(account: &str) -> bool {
        if account.len() > 10 || account.len() < 9 {
            return false;
        }
        let regex = regex!(r"(\d{9})|(\d{6}[YGHE\d]\d{3})");
        return regex.is_match(&account.to_uppercase());
    }
}

#[test]
fn test_username_validator() {
    use crate::model::user::Validator;

    assert!(Validator::validate_username("1812100505"));
    assert!(Validator::validate_username("1910100110"));
    assert!(Validator::validate_username("181042Y109"));
    assert!(!Validator::validate_username("19101001100"));
    assert!(Validator::validate_username("191010011"));
    assert!(!Validator::validate_username("19101001"));
    assert!(!Validator::validate_username("181042Q109"))
}

pub async fn query(pool: &PgPool, account: &str) -> Result<Option<User>> {
    // 在数据库中查询信息
    let user = sqlx::query_as(
        "SELECT uid, account, create_time, role, is_block FROM user.account WHERE username = $1 LIMIT 1;",
    )
    .bind(account)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn create(pool: &PgPool, account: &str) -> Result<User> {
    let user: User = sqlx::query_as(
        "INSERT INTO user.account (account) VALUES($1) RETURNING (uid, account, create_time, role, is_block);",
    )
    .bind(account)
    .fetch_one(pool)
    .await?;
    Ok(user)
}
