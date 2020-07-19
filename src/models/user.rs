//! This module provides the ability to create, update and delete users including authentication tokens.

use crate::error::{ApiError, Result};
use actix_http::http::StatusCode;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
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
    Forbidden = 5,
    #[fail(display = "凭据无效")]
    LoginFailed = 6,
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
    pub async fn password_login(&self, client: &PgPool) -> Result<Person> {
        let user: Option<Person> = sqlx::query_as(
            "SELECT p.uid, nick_name, avatar, is_disabled, is_admin, country, province, city, language, create_time
                FROM public.person p
                RIGHT JOIN authentication auth on p.uid = auth.uid
                WHERE auth.login_type = 1 AND auth.account = $2 AND auth.credential = $3"
        )
            .bind(&self.account)
            .bind(&self.credential)
            .fetch_optional(client)
            .await?;
        match user {
            Some(user) => Ok(user),
            None => Err(ApiError::new(UserError::LoginFailed)),
        }
    }

    pub async fn wechat_login(&self, client: &PgPool) -> Result<Person> {
        let user: Option<Person> = sqlx::query_as(
            "SELECT p.uid, nick_name, avatar, is_disabled, is_admin, country, province, city, language, create_time
                FROM public.person p
                RIGHT JOIN authentication auth on p.uid = auth.uid
                WHERE auth.login_type = 0 AND auth.account = $1"
        )
            .bind(&self.account)
            .fetch_optional(client)
            .await?;
        match user {
            Some(user) => Ok(user),
            None => Err(ApiError::new(UserError::LoginFailed)),
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

    pub async fn query_by_uid(client: &PgPool, uid: i32) -> Result<Option<Person>> {
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

    /// Get identity info
    pub async fn get_identity(client: &PgPool, uid: i32) -> Result<Option<Identity>> {
        let identity: Option<Identity> = sqlx::query_as(
            "SELECT uid, realname, student_id, oa_secret, oa_certified, identity_number, realname
            FROM public.identities WHERE uid = $1",
        )
        .bind(uid)
        .fetch_optional(client)
        .await?;
        Ok(identity)
    }

    /// Set identity info
    pub async fn set_identity(client: &PgPool, uid: i32, identity: &Identity) -> Result<()> {
        if let Some(id_number) = &identity.identity_number {
            if !Identity::validate_identity_number(id_number.as_bytes()) {
                return Err(ApiError::new(UserError::InvalidIdNumber));
            }
        }
        if let Some(oa_secret) = &identity.oa_secret {
            if !Identity::validate_oa_account(&identity.student_id, oa_secret).await? {
                return Err(ApiError::new(UserError::OaSecretFailed));
            }
        }
        let _ = sqlx::query(
            "INSERT INTO public.identities (uid, realname, student_id, oa_secret, oa_certified, identity_number)
                VALUES ($1, $2, $3, $4, true, $5)
                ON CONFLICT (uid)
                DO UPDATE SET oa_secret = $4, oa_certified = true, identity_number = $5;")
            .bind(uid)
            .bind(&identity.realname)
            .bind(&identity.student_id)
            .bind(&identity.oa_secret)
            .bind(&identity.identity_number)
            .execute(client)
            .await?;
        Ok(())
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

/// User real name and other personal information.
#[derive(Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
pub struct Identity {
    /// Person uid
    pub uid: i32,
    /// Real name
    pub realname: String,
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

async fn oa_password_check(account: &String, password: &String) -> Result<bool> {
    use actix_web::client;

    #[derive(Serialize)]
    struct RequestPayload {
        pub code: String,
        pub pwd: String,
    }
    if let Ok(mut web_client) = client::Client::new()
        .post("http://210.35.96.114/report/report/ssoCheckUser")
        .set_header("Referer", "http://xgfy.sit.edu.cn/h5/")
        .send_json(&RequestPayload {
            code: account.clone(),
            pwd: password.clone(),
        })
        .await
    {
        if web_client.status() == StatusCode::OK {
            let body = web_client.body().await?;
            return Ok(body.as_ref() == r#"{"code":0,"msg":null,"data":true}"#.as_bytes());
        }
    }
    Err(ApiError::new(UserError::OaNetworkFailed))
}

impl Identity {
    pub fn new(uid: i32, student_id: &String) -> Self {
        Self {
            uid,
            student_id: student_id.clone(),
            ..Identity::default()
        }
    }

    pub async fn validate_oa_account(student_id: &String, oa_secret: &String) -> Result<bool> {
        oa_password_check(student_id, oa_secret).await
    }

    pub fn validate_identity_number(identity_number: &[u8]) -> bool {
        let magic_array = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
        let tail_chars = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
        let mut sum: usize = 0;

        if identity_number.len() != 18 {
            return false;
        }
        for i in 0..17 {
            sum += magic_array[i] as usize * (identity_number[i] - '0' as u8) as usize;
        }
        return identity_number[17] as char == (if sum % 11 != 2 { tail_chars[sum % 11] } else { 'X' });
    }
}

mod test {
    #[test]
    pub fn test_identity_number_validation() {
        assert_eq!(
            true,
            super::Identity::validate_identity_number("110101192007156996".as_bytes())
        );
        assert_eq!(
            false,
            super::Identity::validate_identity_number("random_string".as_bytes())
        );
        assert_eq!(
            true,
            super::Identity::validate_identity_number("210202192007159834".as_bytes())
        );
    }
}
