use chrono::Utc;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::error::Result;
use crate::models::user::LOGIN_BY_CAMPUS_WEB;

use super::{Authentication, Identity, Person, UserError};
use super::{LOGIN_BY_PASSWORD, LOGIN_BY_WECHAT};

impl Authentication {
    pub fn from_password(username: String, password: String) -> Self {
        Authentication {
            uid: 0,
            login_type: LOGIN_BY_PASSWORD,
            account: username,
            credential: Some(password),
        }
    }

    pub fn from_wechat(open_id: &str) -> Self {
        Authentication {
            uid: 0,
            login_type: LOGIN_BY_WECHAT,
            account: open_id.to_string(),
            credential: None,
        }
    }

    pub fn from_campus_auth(account: String, password: String) -> Self {
        Authentication {
            uid: 0,
            login_type: LOGIN_BY_CAMPUS_WEB,
            account,
            credential: Some(password),
        }
    }
    /// Login by a password, return a Person structure if success. Otherwise, an UserError will be returned.
    pub async fn password_login(&self, client: &PgPool) -> Result<Person> {
        let user: Option<Person> = sqlx::query_as(
            "SELECT p.uid, nick_name, avatar, is_disabled, is_admin, gender, country, province, city, language, create_time
                FROM public.person p
                RIGHT JOIN authentication auth on p.uid = auth.uid
                WHERE auth.login_type = 1 AND auth.account = $1 AND auth.credential = $2 LIMIT 1"
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
            "SELECT p.uid, nick_name, avatar, is_disabled, is_admin, gender, country, province, city, language, create_time
                FROM public.person p
                RIGHT JOIN authentication auth on p.uid = auth.uid
                WHERE auth.login_type = 0 AND auth.account = $1 LIMIT 1"
        )
            .bind(&self.account)
            .fetch_optional(client)
            .await?;
        match user {
            Some(user) => Ok(user),
            None => Err(ApiError::new(UserError::LoginFailed)),
        }
    }

    pub async fn campus_login(&mut self, client: &PgPool) -> Result<Person> {
        let auth: Option<Authentication> = sqlx::query_as(
            "SELECT uid, login_type, account, credential FROM public.authentication
                WHERE login_type = 2 AND account = $1 LIMIT 1",
        )
        .bind(&self.account)
        .fetch_optional(client)
        .await?;

        // If campus credential filled in database, return login success, even if the credential is
        // out of date. Because password error will be thrown out if backend function failed, when,
        // for example, loading course data...
        let auth = auth.ok_or_else(|| ApiError::new(UserError::LoginFailed))?;
        self.uid = auth.uid;

        if auth.credential.eq(&self.credential) && auth.credential.is_some() {
            return Person::get(client, auth.uid).await;
        }
        Err(ApiError::new(UserError::LoginFailed))
    }

    pub async fn campus_update(&self, client: &PgPool) -> Result<()> {
        sqlx::query(
            "UPDATE public.authentication SET credential = $1
                WHERE login_type = 2 AND account = $2",
        )
        .bind(&self.credential)
        .bind(&self.account)
        .execute(client)
        .await?;
        Ok(())
    }
}

impl Person {
    pub fn new() -> Self {
        Person::default()
    }

    /// Bind authentication, if auth type already exists, this function will override the old record.
    pub async fn update_authentication(&self, client: &PgPool, auth: &Authentication) -> Result<()> {
        // Note: Alter username is not allowed.
        let _ = sqlx::query(
            "INSERT INTO
                    authentication (uid, login_type, account, credential) VALUES ($1, $2, $3, $4)
                ON CONFLICT (uid, login_type)
                DO UPDATE SET credential = $4",
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

    pub async fn update(&self, client: &PgPool) -> Result<()> {
        sqlx::query(
            "UPDATE public.person SET gender = $1, country = $2, province = $3, city = $4, avatar = $5\
                WHERE uid = $6",
        )
        .bind(self.gender)
        .bind(&self.country)
        .bind(&self.province)
        .bind(&self.city)
        .bind(&self.avatar)
        .bind(self.uid)
        .execute(client)
        .await?;
        Ok(())
    }

    pub async fn list(client: &PgPool, page_index: u32, page_size: u32) -> Result<Vec<Self>> {
        let users: Vec<Person> = sqlx::query_as(
            "SELECT uid, nick_name, avatar, is_disabled, is_admin, gender, country, province, city, language, create_time
                 FROM public.person LIMIT $1 OFFSET $2")
            .bind(page_size as i32)
            .bind(((page_index - 1) * page_size) as i32)
            .fetch_all(client)
            .await?;
        Ok(users)
    }

    pub async fn get(client: &PgPool, uid: i32) -> Result<Person> {
        let user: Option<Person> = sqlx::query_as(
            "SELECT uid, nick_name, avatar, is_disabled, is_admin, gender, country, province, city, language, create_time
                FROM public.person WHERE uid = $1 LIMIT 1",
        )
            .bind(uid)
            .fetch_optional(client)
            .await?;
        user.ok_or_else(|| ApiError::new(UserError::NoSuchUser))
    }

    pub async fn fuzzy_query(
        client: &PgPool,
        query_string: &str,
        page_index: u32,
        count: u32,
    ) -> Result<Vec<Person>> {
        let users: Vec<Person> = sqlx::query_as(
            "SELECT nick_name, avatar, is_disabled, is_admin, gender, country, province, city, language, create_time
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
            "SELECT uid, real_name, student_id, oa_secret, oa_certified, identity_number
            FROM public.identities WHERE uid = $1",
        )
        .bind(uid)
        .fetch_optional(client)
        .await?;
        Ok(identity)
    }

    /// Set identity info
    pub async fn set_identity(&self, client: &PgPool, identity: &mut Identity) -> Result<()> {
        if let Some(id_number) = &identity.identity_number {
            if !Identity::validate_identity_number(id_number.as_str()) {
                return Err(ApiError::new(UserError::InvalidIdNumber));
            }
        }
        if let Some(oa_secret) = &identity.oa_secret {
            // Throw UserError::OaSecretFailed if password is wrong.
            Identity::validate_oa_account(&identity.student_id, oa_secret).await?;
            identity.oa_certified = true;
        }
        let _ = sqlx::query(
            "INSERT INTO public.identities (uid, real_name, student_id, oa_secret, oa_certified, identity_number)
                VALUES ($1, $2, $3, $4, true, $5)
                ON CONFLICT (uid)
                DO UPDATE SET oa_secret = $4, oa_certified = true, identity_number = $5;")
            .bind(self.uid)
            .bind(&identity.real_name)
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
            gender: 0,
            country: None,
            province: None,
            city: None,
            language: None,
            create_time: Utc::now().naive_local(),
        }
    }
}
