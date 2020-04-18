use actix_web::{Error, HttpResponse, post, ResponseError, web};
use deadpool_postgres::{ClientWrapper, Pool};
use serde::{Deserialize, Serialize};

use crate::error::{Result, UserError};
use crate::error::ServerError;
use crate::jwt::{encode_jwt, JwtClaims};
use crate::user;
use crate::user::actions::{User, UserExtra};
use crate::user::NormalResponse;
use crate::user::wechat::WxSession;

#[derive(Debug, Deserialize)]
pub struct AuthStru {
    // loginType. Defined in user/models.rs, value can be either LOGIN_WECHAT
    // or LOGIN_USERNAME
    #[serde(rename = "loginType")]
    login_type: i32,
    // The code that provided by wechat wx.login()
    #[serde(rename = "wxCode")]
    wechat_code: Option<String>,
    // Used in LOGIN_USERNAME, username
    account: Option<String>,
    // Used in LOGIN_USERNAME, password
    credential: Option<String>,
}


#[post("/session")]
pub async fn login(
    pool: web::Data<Pool>,
    form: web::Form<AuthStru>
) -> Result<HttpResponse> {
    let conn = pool.get().await?;
    let parameters: AuthStru = form.into_inner();
    let uid;

    // 对应不同登录方式，验证登录凭据，获取用户ID（uid）
    match parameters {
        // 用户名密码方式登录
        AuthStru {
            login_type: LOGIN_USERNAME,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            uid = user::actions::login(&conn, username, password).await?;
        }
        // 微信方式登录
        AuthStru {
            login_type: LOGIN_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            // 微信登录流程
            // 用前端提供的临时动态字符串换取 session_key 和用户的 openid
            // 然后在数据库查询
            let wechat_token: WxSession = user::wechat::get_session_by_code(wechat_code.as_str()).await?;
            uid = user::actions::wechat_login(&conn, wechat_token.openid).await?;
        }
        _ => {
            return Err(ServerError::from(UserError::BadParameter));
        }
    }
    // 生成 JWT 字符串并返回
    #[derive(Serialize)]
    struct LoginResponse {
        token: String,
    }

    let resp = LoginResponse { token: encode_jwt(&JwtClaims { uid })? };
    Ok(HttpResponse::Ok().body(NormalResponse::new(resp).to_string()))
}

#[post("/user")]
pub async fn create_user(
    pool: web::Data<Pool>,
    form: web::Form<UserExtra>,
) -> Result<HttpResponse> {
    let conn = pool.get().await?;
    let parameters: UserExtra = form.into_inner();

    let nick_name = parameters.nick_name.clone().unwrap_or(String::from(""));
    let uid = user::actions::create_user(&conn, nick_name, parameters).await?;
    #[derive(Serialize)]
    struct CreateResponse {
        uid: i32,
    }

    let resp = CreateResponse { uid };
    Ok(HttpResponse::Ok().body(NormalResponse::new(resp).to_string()))
}

#[post("/user/{uid}/authentication")]
pub async fn bind_authentication(
    pool: web::Data<Pool>,
    form: web::Form<AuthStru>,
    req: web::HttpRequest,
) -> Result<HttpResponse> {
    let conn = pool.get().await?;
    let parameters: AuthStru = form.into_inner();

    // 参数不存在或不是数字，均返回参数错误 (BadParameter)
    let uid = req.match_info().get("uid").ok_or(ServerError::from(UserError::BadParameter))?;
    let uid: i32 = uid.parse().map_err(|_| ServerError::from(UserError::BadParameter))?;

    match parameters {
        AuthStru {
            login_type: LOGIN_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            let wechat_token: WxSession = user::wechat::get_session_by_code(wechat_code.as_str()).await?;
            user::actions::bind_wechat(&conn, uid, wechat_token.openid).await?;
        },
        AuthStru {
            login_type: LOGIN_USERNAME,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            user::actions::bind_password(&conn, uid, username, password).await?;
        },
        _ => {
            return Err(ServerError::from(UserError::BadParameter));
        }
    }
    #[derive(Serialize)]
    struct EmptyReponse;
    Ok(HttpResponse::Ok().body(NormalResponse::new(EmptyReponse).to_string()))
}