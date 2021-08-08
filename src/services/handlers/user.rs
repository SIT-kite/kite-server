use actix_web::{get, post, put, web, HttpResponse};
use serde::{Deserialize, Serialize};
use wechat_sdk::wechat::{Login, WxSession};

use crate::error::{ApiError, Result};
use crate::jwt::encode_jwt;
use crate::models::file::AvatarManager;
use crate::models::user::{get_default_avatar, Authentication, Identity, Person, UserError};
use crate::models::user::{LOGIN_BY_CAMPUS_WEB, LOGIN_BY_PASSWORD, LOGIN_BY_WECHAT};
use crate::models::CommonError;
use crate::services::{response::ApiResponse, AppState, JwtToken};

#[derive(Debug, Deserialize)]
pub struct AuthParameters {
    // Can be either _LOGIN_BY_WECHAT or _LOGIN_BY_PASSWORD
    #[serde(rename = "loginType")]
    login_type: i32,
    // The code that provided by wechat wx.login()
    #[serde(rename = "wxCode")]
    wechat_code: Option<String>,
    // Used in _LOGIN_BY_PASSWORD, username
    account: Option<String>,
    // Used in _LOGIN_BY_PASSWORD, password
    credential: Option<String>,
}

#[post("/session")]
pub async fn login(app: web::Data<AppState>, form: web::Form<AuthParameters>) -> Result<HttpResponse> {
    let parameters: AuthParameters = form.into_inner();
    let user: Person;

    match parameters {
        // Login by username / password.
        AuthParameters {
            login_type: LOGIN_BY_PASSWORD,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            let auth: Authentication = Authentication::from_password(username, password);
            user = auth.password_login(&app.pool).await?;
        }
        // Login by wechat.
        AuthParameters {
            login_type: LOGIN_BY_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            let wechat_token: WxSession = app.wx_client.code2session(&wechat_code).await?;
            let auth: Authentication = Authentication::from_wechat(&wechat_token.openid);
            user = auth.wechat_login(&app.pool).await?;
        }
        // Login by campus web (student id and password).
        AuthParameters {
            login_type: LOGIN_BY_CAMPUS_WEB,
            account: Some(account),
            credential: Some(password),
            ..
        } => {
            let mut auth = Authentication::from_campus_auth(account.clone(), password.clone());
            let result = auth.campus_login(&app.pool).await;

            match result {
                Ok(u) => user = u,
                Err(e) => {
                    Identity::validate_oa_account(&account, &password).await?;
                    // If password failed, verify on auth-server to update.
                    if e == ApiError::new(UserError::LoginFailed) {
                        auth.campus_update(&app.pool).await?;
                        user = auth.campus_login(&app.pool).await?;
                    } else {
                        return Err(e);
                    }
                }
            };
        }
        _ => {
            return Err(ApiError::new(CommonError::Parameter));
        }
    }
    if user.is_disabled {
        return Err(ApiError::new(UserError::Disabled));
    }

    #[derive(Serialize)]
    struct LoginResponse {
        token: String,
        data: Person,
    }
    let token = encode_jwt(&JwtToken {
        uid: user.uid,
        is_admin: user.is_admin,
    })?;
    let resp = LoginResponse { token, data: user };
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(resp)))
}

#[derive(Deserialize)]
pub struct ListUsers {
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub index: Option<u32>,
}

#[get("/user")]
pub async fn list_users(app: web::Data<AppState>, form: web::Query<ListUsers>) -> Result<HttpResponse> {
    let parameter = form.into_inner();
    let userlist = Person::list(
        &app.pool,
        parameter.index.unwrap_or(1),
        parameter.page_size.unwrap_or(20),
    )
    .await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(userlist)))
}

#[derive(Deserialize)]
pub struct SubmittedPerson {
    /// Nickname. For users uses wechat to register, use wehcat name by default.
    #[serde(rename = "nickName")]
    pub nick_name: Option<String>,
    /// User avatar url.
    #[serde(rename = "avatarUrl")]
    pub avatar: Option<String>,
    /// Country from wechat
    pub country: Option<String>,
    /// Province from wechat.
    pub province: Option<String>,
    pub city: Option<String>,
    /// Language code, like zh-cn
    pub language: Option<String>,
}

#[post("/user")]
pub async fn create_user(
    app: web::Data<AppState>,
    form: web::Form<SubmittedPerson>,
) -> Result<HttpResponse> {
    let parameters: SubmittedPerson = form.into_inner();
    let mut user: Person = Person::new();

    if parameters.nick_name.is_none() {
        return Err(ApiError::new(CommonError::Parameter));
    }
    user.nick_name = parameters.nick_name.unwrap();
    user.country = parameters.country;
    user.province = parameters.province;
    user.city = parameters.city;
    user.language = parameters.language;

    if let Some(avatar_url) = parameters.avatar {
        let avatar_storage = AvatarManager::new(&app.pool);

        user.avatar = avatar_storage
            .save(0, &avatar_url)
            .await?
            .url
            .unwrap_or_else(|| get_default_avatar().to_string());
    }
    user.register(&app.pool).await?;

    #[derive(Serialize)]
    struct CreateResponse {
        uid: i32,
        token: String,
    }

    let resp = CreateResponse {
        uid: user.uid,
        token: encode_jwt(&JwtToken {
            uid: user.uid,
            is_admin: user.is_admin,
        })?,
    };
    Ok(HttpResponse::Ok().body(ApiResponse::normal(resp).to_string()))
}

#[put("/user/{uid}")]
pub async fn update_user_detail(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    form: web::Form<SubmittedPerson>,
    uid: web::Path<i32>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let uid = uid.into_inner();

    if !token.is_admin && uid != token.uid {
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let mut person = Person::get(&app.pool, uid).await?;
    let form = form.into_inner();

    if let Some(nick_name) = form.nick_name {
        person.nick_name = nick_name;
    }
    if let Some(city) = form.city {
        person.city = Some(city);
    }
    if let Some(province) = form.province {
        person.province = Some(province);
    }
    if let Some(country) = form.country {
        person.country = Some(country);
    }
    if let Some(avatar_url) = form.avatar {
        let avatar_storage = AvatarManager::new(&app.pool);
        let stored_avatar = avatar_storage.query(&avatar_url).await;
        let final_url = match stored_avatar {
            // Use stored avatar
            Ok(a) => a.url,
            // Download and save avatar if not stored.
            Err(_) => avatar_storage.save(0, &avatar_url).await?.url,
        };
        person.avatar = final_url.unwrap_or_else(|| get_default_avatar().to_string());
    }
    person.update(&app.pool).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::normal(person)))
}

#[post("/user/{uid}/authentication")]
pub async fn bind_authentication(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    form: web::Form<AuthParameters>,
    uid: web::Path<i32>,
) -> Result<HttpResponse> {
    let parameters: AuthParameters = form.into_inner();
    let token = token.unwrap();
    let uid = uid.into_inner();

    if token.uid != uid && !token.is_admin {
        return Err(ApiError::new(CommonError::Parameter));
    }
    let user = Person::get(&app.pool, uid).await?;

    match parameters {
        AuthParameters {
            login_type: LOGIN_BY_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            let wechat_token: WxSession = app.wx_client.code2session(&wechat_code).await?;
            let auth: Authentication = Authentication::from_wechat(&wechat_token.openid);
            user.update_authentication(&app.pool, &auth).await?;
        }
        AuthParameters {
            login_type: LOGIN_BY_PASSWORD,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            // Patch: Ordinary users are not allowed to log in with a password,
            // so as to prevent abuse of the interface.
            if !token.is_admin {
                return Err(ApiError::new(UserError::AuthTypeNotAllowed));
            }
            let auth = Authentication::from_password(username, password);
            user.update_authentication(&app.pool, &auth).await?;
        }
        AuthParameters {
            login_type: LOGIN_BY_CAMPUS_WEB,
            account: Some(account),
            credential: Some(password),
            ..
        } => {
            let auth = Authentication::from_campus_auth(account.clone(), password.clone());

            Identity::validate_oa_account(&account, &password).await?;
            user.update_authentication(&app.pool, &auth).await?;
        }
        _ => {
            return Err(ApiError::new(CommonError::Parameter));
        }
    }
    Ok(HttpResponse::Ok().json(ApiResponse::empty()))
}

#[get("/user/{uid}")]
pub async fn get_user_detail(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    uid: web::Path<i32>,
) -> Result<HttpResponse> {
    if token.is_none() {
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let token = token.unwrap();
    let uid = uid.into_inner();

    if uid != token.uid && !token.is_admin {
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let user = Person::get(&app.pool, uid).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(&user)))
}

#[get("/user/{uid}/identity")]
pub async fn get_user_identity(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    uid: web::Path<i32>,
) -> Result<HttpResponse> {
    let uid = uid.into_inner();
    let token = token.unwrap();

    if token.uid != uid && !token.is_admin {
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let identity = Person::get_identity(&app.pool, uid).await?;
    if identity.is_none() {
        return Err(ApiError::new(UserError::NoSuchUser));
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(identity.unwrap())))
}

#[derive(Deserialize)]
pub struct IdentityPost {
    /// Real name
    #[serde(rename = "realName")]
    pub real_name: String,
    /// Student id
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// OA secret(password)
    #[serde(rename = "oaSecret")]
    pub oa_secret: Option<String>,
    /// ID card number
    #[serde(rename = "identityNumber")]
    pub identity_number: Option<String>,
}

#[post("/user/{uid}/identity")]
pub async fn set_user_identity(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    uid: web::Path<i32>,
    data: web::Form<IdentityPost>,
) -> Result<HttpResponse> {
    let uid = uid.into_inner();
    let token = token.unwrap();

    if token.uid != uid && !token.is_admin {
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let identity_post = data.into_inner();
    let mut identity = Identity {
        uid,
        real_name: identity_post.real_name,
        student_id: identity_post.student_id,
        oa_secret: identity_post.oa_secret,
        oa_certified: false,
        identity_number: identity_post.identity_number,
    };
    let person = Person::get(&app.pool, uid).await?;
    person.set_identity(&app.pool, &mut identity).await?;

    if identity.oa_certified {
        let auth = Authentication::from_campus_auth(
            identity.student_id.clone(),
            identity.oa_secret.unwrap_or_default(),
        );
        // Update authentication approach so that students can log in by student-id and password in campus network.
        person.update_authentication(&app.pool, &auth).await?;
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}
