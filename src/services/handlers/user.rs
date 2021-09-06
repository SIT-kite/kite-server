use crate::bridge::{PortalAuthRequest, PortalAuthResponse};
use crate::error::{ApiError, Result};
use crate::jwt::encode_jwt;
use crate::models::file::AvatarManager;
use crate::models::user::{get_default_avatar, Authentication, Identity, Person, UserError};
use crate::models::user::{LOGIN_BY_PASSWORD, LOGIN_BY_WECHAT};
use crate::models::CommonError;
use crate::services::{response::ApiResponse, AppState, JwtToken};
use actix_web::{get, post, put, web, HttpResponse};
use serde::Deserialize;
use wechat_sdk::wechat::{Login, WxSession};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthParameters {
    // Can be either _LOGIN_BY_WECHAT or _LOGIN_BY_PASSWORD
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
        _ => {
            return Err(ApiError::new(CommonError::Parameter));
        }
    }
    if user.is_disabled {
        return Err(ApiError::new(UserError::Disabled));
    }

    let token = encode_jwt(&JwtToken {
        uid: user.uid,
        is_admin: user.is_admin,
    })?;
    let response = serde_json::json!({
        "token": token,
        "data": user,
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsers {
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
#[serde(rename_all = "camelCase")]
pub struct SubmittedPerson {
    /// Nickname. For users uses wechat to register, use wechat name by default.
    pub nick_name: Option<String>,
    /// User avatar url.
    pub avatar_url: Option<String>,
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

    /* Necessary items. */
    user.nick_name = parameters.nick_name.unwrap_or_default();
    user.avatar = parameters.avatar_url.unwrap_or_default();
    if user.nick_name.is_empty() || !user.avatar.starts_with("https://") {
        return Err(ApiError::new(CommonError::Parameter));
    }

    /* Optional parameters. */
    user.country = parameters.country;
    user.province = parameters.province;
    user.city = parameters.city;
    user.language = parameters.language;

    let avatar_storage = AvatarManager::new(&app.pool);
    user.avatar = avatar_storage
        .save(0, &user.avatar)
        .await?
        .url
        .unwrap_or_else(|| get_default_avatar().to_string());
    user.register(&app.pool).await?;

    let response = serde_json::json!({
        "uid": user.uid,
        "token": encode_jwt(&JwtToken {
            uid: user.uid,
            is_admin: user.is_admin,
        })?
    });

    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
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
        if nick_name.is_empty() {
            return Err(ApiError::new(CommonError::Parameter));
        }
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
    if let Some(avatar_url) = form.avatar_url {
        if !avatar_url.starts_with("https://") {
            return Err(ApiError::new(CommonError::Parameter));
        }

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

    Person::get_identity(&app.pool, uid)
        .await?
        .map(|i| HttpResponse::Ok().json(&ApiResponse::normal(i)))
        .ok_or_else(|| ApiError::new(UserError::NoSuchUser))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityPost {
    /// Student id
    pub student_id: String,
    /// OA secret(password)
    pub oa_secret: String,
}

#[post("/user/{uid}/identity")]
pub async fn set_user_identity(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    uid: web::Path<i32>,
    data: web::Form<IdentityPost>,
) -> Result<HttpResponse> {
    let uid = uid.into_inner();
    let token = token.ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    if token.uid != uid && !token.is_admin {
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let identity_post = data.into_inner();
    let mut identity = Identity {
        uid,
        student_id: identity_post.student_id,
        oa_secret: identity_post.oa_secret,
        oa_certified: false,
    };
    let person = Person::get(&app.pool, uid).await?;
    person.set_identity(&app.pool, &mut identity, &app.agents).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}
