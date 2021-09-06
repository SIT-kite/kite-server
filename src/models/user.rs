//! This module provides the ability to create, update and delete users including authentication tokens.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub use person::get_default_avatar;

mod authserver;
mod identity;
mod person;

/* Constants at the edge between self and database. */

/// Login Type.
pub const LOGIN_BY_WECHAT: i32 = 0;
pub const LOGIN_BY_PASSWORD: i32 = 1;

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum UserError {
    #[error("账户已禁用")]
    Disabled = 50,
    #[error("凭据无效")]
    LoginFailed = 51,
    #[error("无法连接校园网完成认证")]
    OaNetworkFailed = 52,
    #[error("OA密码认证失败")]
    OaSecretFailed = 53,
    #[error("错误的身份证号码")]
    InvalidIdNumber = 54,
    #[error("普通用户不允许使用用户名密码登录")]
    AuthTypeNotAllowed = 55,
    #[error("找不到用户")]
    NoSuchUser = 56,
    #[error("请修改默认OA密码")]
    DefaultSecretDenied = 57,
    #[error("学号格式不正确")]
    NoSuchStudentNo = 58,
}

/* Models */

/// Authentication structure, similar to table "authentication" in database.
/// Record everybody's login credentials.
#[derive(Default, sqlx::FromRow)]
pub struct Authentication {
    /// Target user.
    pub uid: i32,
    /// login type.
    pub login_type: i32,
    /// Username or wechat token (id).
    pub account: String,
    /// Password if uses username.
    pub credential: Option<String>,
}

/// Base information of each account.
#[derive(sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    /// Target user, key.
    pub uid: i32,
    /// Nickname. For users uses wechat to register, use wechat name by default.
    pub nick_name: String,
    /// User avatar url.
    pub avatar: String,
    /// Is disabled. False by default.
    #[serde(skip_serializing)]
    pub is_disabled: bool,
    /// Is administrator. False by default.
    pub is_admin: bool,
    /// Gender. 0 for unknown, 1 for male, 2 for female.
    pub gender: i16,
    /// Country from wechat
    pub country: Option<String>,
    /// Province from wechat.
    pub province: Option<String>,
    pub city: Option<String>,
    /// Language code, like zh-cn
    #[serde(skip_serializing)]
    pub language: Option<String>,
    /// Account create time.
    pub create_time: NaiveDateTime,
}

/// User real name and other personal information.
#[derive(Default, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    /// Person uid
    pub uid: i32,
    /// Student id
    pub student_id: String,
    /// OA secret(password)
    pub oa_secret: String,
    /// Whether OA certified or not
    pub oa_certified: bool,
}
