use crate::error::{Result, ServerError};
use crate::jwt::encode_jwt;
use crate::server::{JwtToken, NormalResponse};
use crate::user::{get_default_avatar, Authentication, Person, UserError};
use crate::user::{_LOGIN_BY_PASSWORD, _LOGIN_BY_WECHAT};
use crate::wechat::{get_session_by_code, WxSession};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
pub async fn login(pool: web::Data<PgPool>, form: web::Form<AuthParameters>) -> Result<HttpResponse> {
    let parameters: AuthParameters = form.into_inner();
    let user: Person;

    match parameters {
        // Login by username / password.
        AuthParameters {
            login_type: _LOGIN_BY_PASSWORD,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            let auth: Authentication = Authentication::from_password(&username, &password);
            user = auth.login(&pool).await?;
        }
        // Login by wechat.
        AuthParameters {
            login_type: _LOGIN_BY_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            let wechat_token: WxSession = get_session_by_code(wechat_code.as_str()).await?;
            let auth: Authentication = Authentication::from_wechat(&wechat_token.openid);
            user = auth.login(&pool).await?;
        }
        _ => {
            return Err(ServerError::new(UserError::BadParameter));
        }
    }
    if user.is_disabled {
        return Err(ServerError::new(UserError::Disabled));
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
    Ok(HttpResponse::Ok().json(&NormalResponse::new(resp)))
}

#[derive(Deserialize)]
pub struct ListUsers {
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub index: Option<u32>,
}

#[get("/user")]
pub async fn list_users(pool: web::Data<PgPool>, form: web::Form<ListUsers>) -> Result<HttpResponse> {
    let parameter = form.into_inner();
    let userlist = Person::list(
        &pool,
        parameter.index.unwrap_or(1),
        parameter.page_size.unwrap_or(20),
    )
    .await?;

    Ok(HttpResponse::Ok().json(&NormalResponse::new(userlist)))
}

#[derive(Deserialize)]
pub struct SubmittedPerson {
    /// Nickname. For users uses wechat to register, use wehcat name by default.
    #[serde(rename = "nickName")]
    pub nick_name: String,
    /// User avatar url.
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
    pool: web::Data<PgPool>,
    form: web::Form<SubmittedPerson>,
) -> Result<HttpResponse> {
    let parameters: SubmittedPerson = form.into_inner();

    let mut user: Person = Person::new();
    user.nick_name = parameters.nick_name;
    user.avatar = parameters.avatar.unwrap_or(get_default_avatar().to_string());
    user.country = parameters.country;
    user.province = parameters.province;
    user.city = parameters.city;
    user.language = parameters.language;

    user.register(&pool).await?;
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
    Ok(HttpResponse::Ok().body(NormalResponse::new(resp).to_string()))
}

#[post("/user/{uid}/authentication")]
pub async fn bind_authentication(
    pool: web::Data<PgPool>,
    form: web::Form<AuthParameters>,
    req: web::HttpRequest,
) -> Result<HttpResponse> {
    let parameters: AuthParameters = form.into_inner();

    let uid: i32 = req
        .match_info()
        .get("uid")
        .and_then(|uid| uid.parse().ok())
        .ok_or(ServerError::new(UserError::BadParameter))?;

    let user = Person::query_uid(&pool, uid)
        .await?
        .ok_or(ServerError::new(UserError::NoSuchUser))?;

    match parameters {
        AuthParameters {
            login_type: _LOGIN_BY_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            let auth = Authentication::from_wechat(&wechat_code);
            user.update_authentication(&pool, &auth).await?;
        }
        AuthParameters {
            login_type: _LOGIN_BY_PASSWORD,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            let auth = Authentication::from_password(&username, &password);
            user.update_authentication(&pool, &auth).await?;
        }
        _ => {
            return Err(ServerError::new(UserError::BadParameter));
        }
    }
    #[derive(Serialize)]
    struct EmptyReponse;
    Ok(HttpResponse::Ok().body(NormalResponse::new(EmptyReponse).to_string()))
}

#[get("/user/{uid}")]
pub async fn get_user_detail(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if let None = token {
        return Err(ServerError::new(UserError::Forbidden));
    }
    let token = token.unwrap();
    let uid_to_query: i32 = req
        .match_info()
        .get("uid")
        .and_then(|uid| uid.parse().ok())
        .ok_or(ServerError::new(UserError::BadParameter))?;
    let uid_operator: i32 = token.uid;

    if uid_operator != uid_to_query && !token.is_admin {
        return Err(ServerError::new(UserError::Forbidden));
    }
    let user = Person::query_uid(&pool, uid_to_query).await?;
    if let None = user {
        return Err(ServerError::new(UserError::NoSuchUser));
    }
    Ok(HttpResponse::Ok().json(&NormalResponse::new(&user.unwrap())))
}
