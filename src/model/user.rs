use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::error::Result;

pub mod role_mask {
    /// 图书馆管理员
    pub const LIBRARY: i32 = 0b10;
    /// 完整权限
    pub const FULL: i32 = 0x7FFFFFFF;
}

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
    #[error("找不到用户")]
    NoSuchUser = 4,
    #[error("凭据认证失败")]
    CredentialFailure = 5,
}

pub struct Validator;

impl Validator {
    pub fn validate_username(account: &str) -> bool {
        if account.len() != 4 && account.len() != 10 && account.len() != 9 {
            return false;
        }
        let regex = regex!(r"^((\d{2}6\d{6})|(\d{4})|(\d{6}[YGHE\d]\d{3}))$");
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
    assert!(!Validator::validate_username("191010011"));
    assert!(!Validator::validate_username("19101001"));
    assert!(!Validator::validate_username("181042Q109"));
    assert!(Validator::validate_username("1234"));
    assert!(Validator::validate_username("0000"));
    assert!(Validator::validate_username("216001234"));
}

pub async fn hit_admin(pool: &PgPool, account: &str, credential: &str) -> Result<bool> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM \"user\".admin WHERE account = $1 AND credential = $2;")
            .bind(account)
            .bind(credential)
            .fetch_one(pool)
            .await?;
    Ok(count != 0)
}

pub async fn get(pool: &PgPool, uid: i32) -> Result<Option<User>> {
    let user = sqlx::query_as(
        "SELECT uid, account, create_time, role, is_block FROM \"user\".account WHERE uid = $1 LIMIT 1;",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn query(pool: &PgPool, account: &str) -> Result<Option<User>> {
    // 在数据库中查询信息
    let user = sqlx::query_as(
        "SELECT uid, account, create_time, role, is_block FROM \"user\".account WHERE account = $1 LIMIT 1;",
    )
    .bind(account)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn create(pool: &PgPool, account: &str) -> Result<User> {
    let user: User = sqlx::query_as(
        "INSERT INTO \"user\".account (account) VALUES($1) \
        ON CONFLICT (account) DO UPDATE SET account = $1 \
        RETURNING uid, account, create_time, role, is_block;",
    )
    .bind(account)
    .fetch_one(pool)
    .await?;
    Ok(user)
}
