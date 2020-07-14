//! This module provides the ability to create, update and delete users including authentication tokens.

use crate::error::{Result, ServerError};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::{postgres::PgQueryAs, PgPool};

/* Constants at the edge between self and database. */

/// Login Type.
pub const _LOGIN_BY_WECHAT: i32 = 0;
pub const _LOGIN_BY_PASSWORD: i32 = 1;

#[derive(Fail, Debug, ToPrimitive)]
pub enum UserError {
    #[fail(display = "参数错误")]
    BadParameter = 2,
    #[fail(display = "权限不足")]
    Forbidden = 4,
    #[fail(display = "凭据无效")]
    LoginFailed = 5,
    #[fail(display = "账户已禁用")]
    Disabled = 6,
    #[fail(display = "未知错误")]
    Unknown = 7,
    #[fail(display = "存在冲突的登录凭据")]
    AuthExists = 15,
    #[fail(display = "找不到用户")]
    NoSuchUser = 16,
}

/* Models */

/// Authentication structure, similar to table "authentication" in database.
/// Record everybody's login credentials.
#[derive(Default, Debug)]
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

impl Authentication {
    pub fn from_password(username: &String, password: &String) -> Self {
        Authentication {
            uid: 0,
            login_type: _LOGIN_BY_PASSWORD,
            account: username.clone(),
            credential: Some(password.clone()),
        }
    }

    pub fn from_wechat(open_id: &String) -> Self {
        Authentication {
            uid: 0,
            login_type: _LOGIN_BY_WECHAT,
            account: open_id.clone(),
            credential: None,
        }
    }
    // Require to verify the credential and login.
    // The function will return an error::Result. When the process success, an i32 value as uid
    // will be returned. Otherwise, a UserError enum, provides the reason.
    pub async fn login(&self, client: &PgPool) -> Result<Person> {
        let user: Option<Person> = sqlx::query_as(
            "SELECT p.uid, nick_name, avatar, is_disabled, is_admin, country, province, city, language, create_time
                FROM public.person p
                RIGHT JOIN authentication auth on p.uid = auth.uid
                WHERE auth.login_type = $1 AND auth.account = $2 AND auth.credential = $3"
        )
            .bind(self.login_type)
            .bind(&self.account)
            .bind(&self.credential)
            .fetch_optional(client)
            .await?;
        match user {
            Some(user) => Ok(user),
            None => Err(ServerError::new(UserError::LoginFailed)),
        }
    }
}

/// Base information of each account.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Person {
    /// Target user, key.
    pub uid: i32,
    /// Nickname. For users uses wechat to register, use wehcat name by default.
    pub nick_name: String,
    /// User avatar url.
    pub avatar: String,
    /// Is disabled. False by default.
    pub is_disabled: bool,
    /// Is administrator. False by default.
    pub is_admin: bool,
    /// Country from wechat
    pub country: Option<String>,
    /// Province from wechat.
    pub province: Option<String>,
    pub city: Option<String>,
    /// Language code, like zh-cn
    pub language: Option<String>,
    /// Account create time.
    #[serde(rename = "createTime")]
    pub create_time: NaiveDateTime,
}

impl Person {
    pub fn new() -> Self {
        Person::default()
    }

    /// Bind authentication, if auth type already exists, this function will override the old record.
    pub async fn update_authentication(&self, client: &PgPool, auth: &Authentication) -> Result<()> {
        let _ = sqlx::query(
            "INSERT INTO
                    authentication (uid, login_type, account, credential) VALUES ($1, $2, $3, $4)
                    ON CONFLICT (uid, login_type)
                    DO UPDATE SET account = $3, credential = $4 WHERE authentication.uid = $1",
        )
        .bind(self.uid)
        .bind(auth.login_type)
        .bind(&auth.account)
        .bind(&auth.credential)
        .execute(client)
        .await?;

        Ok(())
    }

    pub async fn register(&mut self, client: &PgPool) -> Result<()> {
        let uid: Option<(i32,)> = sqlx::query_as(
            "INSERT INTO public.person
                (nick_name, avatar, country, province, city, language, create_time)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING uid",
        )
        .bind(&self.nick_name)
        .bind(&self.avatar)
        .bind(&self.country)
        .bind(&self.province)
        .bind(&self.city)
        .bind(&self.language)
        .bind(&self.create_time)
        .fetch_optional(client)
        .await?;
        if let Some((uid_value,)) = uid {
            self.uid = uid_value;
        }
        // TODO: update code here.
        Ok(())
    }

    pub async fn list(client: &PgPool, page_index: u32, page_size: u32) -> Result<Vec<Self>> {
        let users: Vec<Person> = sqlx::query_as(
            "SELECT uid, nick_name, avatar, is_disabled, is_admin, country, province, city, language, create_time
                 FROM public.person LIMIT $1 OFFSET $2")
            .bind(page_size as i32)
            .bind(((page_index - 1) * page_size) as i32)
            .fetch_all(client)
            .await?;
        Ok(users)
    }

    pub async fn query_uid(client: &PgPool, uid: i32) -> Result<Option<Person>> {
        let user: Option<Person> = sqlx::query_as(
            "SELECT uid, nick_name, avatar, is_disabled, is_admin, country, province, city, language, create_time
                FROM public.person WHERE uid = $1 LIMIT 1",
        )
            .bind(uid)
            .fetch_optional(client)
            .await?;
        Ok(user)
    }

    pub async fn fuzzy_query(
        client: &PgPool,
        query_string: &String,
        page_index: u32,
        count: u32,
    ) -> Result<Vec<Person>> {
        let users: Vec<Person> = sqlx::query_as(
            "SELECT nick_name, avatar, is_disabled, is_admin, country, province, city, language, create_time
                FROM public.person WHERE nick_name = $1
                LIMIT $2 OFFSET $3",
        )
            .bind(query_string)
            .bind(count)
            .bind((page_index - 1) * count)
            .fetch_all(client)
            .await?;
        Ok(users)
    }
}

/// Default avatar for new user.
pub fn get_default_avatar() -> &'static str {
    "https://kite.sunnysab.cn/static/icon.png"
}

impl Default for Person {
    fn default() -> Self {
        Person {
            uid: 0,
            nick_name: "".to_string(),
            avatar: get_default_avatar().to_string(),
            is_disabled: false,
            is_admin: false,
            country: None,
            province: None,
            city: None,
            language: None,
            create_time: Utc::now().naive_local(),
        }
    }
}
