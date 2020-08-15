//! This module provides the ability to create, update and delete users including authentication tokens.

mod authserver;
mod identity;
mod person;
pub(crate) mod wechat;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub use person::get_default_avatar;

/* Constants at the edge between self and database. */

/// Login Type.
pub const LOGIN_BY_WECHAT: i32 = 0;
pub const LOGIN_BY_PASSWORD: i32 = 1;

#[derive(Fail, Debug, ToPrimitive)]
pub enum UserError {
    #[fail(display = "账户已禁用")]
    Disabled = 50,
    #[fail(display = "找不到用户")]
    NoSuchUser = 51,
    #[fail(display = "无法连接校园网完成认证")]
    OaNetworkFailed = 52,
    #[fail(display = "OA密码认证失败")]
    OaSecretFailed = 53,
    #[fail(display = "错误的身份证号码")]
    InvalidIdNumber = 54,
    #[fail(display = "普通用户不允许使用用户名密码登录")]
    AuthTypeNotAllowed = 55,
    #[fail(display = "凭据无效")]
    LoginFailed = 56,
}

/* Models */

/// Authentication structure, similar to table "authentication" in database.
/// Record everybody's login credentials.
#[derive(Default)]
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
pub struct Person {
    /// Target user, key.
    pub uid: i32,
    /// Nickname. For users uses wechat to register, use wechat name by default.
    #[serde(rename = "nickName")]
    pub nick_name: String,
    /// User avatar url.
    pub avatar: String,
    /// Is disabled. False by default.
    #[serde(skip_serializing)]
    pub is_disabled: bool,
    /// Is administrator. False by default.
    #[serde(rename = "isAdmin")]
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
    #[serde(rename = "createTime")]
    pub create_time: NaiveDateTime,
}

/// User real name and other personal information.
#[derive(Default, Serialize, Deserialize, sqlx::FromRow)]
pub struct Identity {
    /// Person uid
    pub uid: i32,
    /// Real name
    #[serde(rename = "realName")]
    pub real_name: String,
    /// Student id
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// OA secret(password)
    #[serde(rename = "oaSecret")]
    pub oa_secret: Option<String>,
    /// Whether OA certified or not
    #[serde(rename = "oaCertified")]
    pub oa_certified: bool,
    /// ID card number
    #[serde(rename = "identityNumber")]
    pub identity_number: Option<String>,
}
