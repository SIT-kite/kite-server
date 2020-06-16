use actix_web::{HttpResponse, post, web};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ServerError, UserError};
use crate::jwt::{encode_jwt, JwtClaims};
use crate::user::{self, wechat::WxSession};

use super::NormalResponse;

pub type Pool<T> = diesel::r2d2::Pool<ConnectionManager<T>>;
pub type PostgresPool = Pool<PgConnection>;

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
pub async fn login(
    pool: web::Data<PostgresPool>,
    form: web::Form<AuthParameters>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    let parameters: AuthParameters = form.into_inner();
    let uid;

    // 对应不同登录方式，验证登录凭据，获取用户 ID（uid）
    match parameters {
        // 用户名密码方式登录
        AuthParameters {
            login_type: _LOGIN_BY_PASSWORD,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            uid = web::block(move || {
                user::password_login(&conn, username.as_ref(), password.as_ref())
            })
            .await
            .map_err(|e| ServerError::from(e.to_string()))?;
        }
        // 微信方式登录
        AuthParameters {
            login_type: _LOGIN_BY_WECHAT,
            wechat_code: Some(wechat_code),
            ..
        } => {
            // 微信登录流程
            // 用前端提供的临时动态字符串换取 session_key 和用户的 openid
            // 然后在数据库查询
            let wechat_token: WxSession =
                user::wechat::get_session_by_code(wechat_code.as_str()).await?;
            uid = web::block(move || user::wechat_login(&conn, wechat_token.openid.as_ref()))
                .await
                .map_err(|e| ServerError::from(e.to_string()))?;
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

    let resp = LoginResponse {
        token: encode_jwt(&JwtClaims { uid })?,
    };
    // Ok(HttpResponse::Ok().body(NormalResponse::new(resp).to_string()))
    Ok(HttpResponse::Ok().json(&NormalResponse::new(resp)))
}

// #[post("/user")]
// pub async fn create_user(
//     pool: web::Data<PostgresPool>,
//     form: web::Form<UserExtra>,
// ) -> Result<HttpResponse> {
//
//     let conn = pool.get();
//     let parameters: UserExtra = form.into_inner();
//
//     let nick_name = parameters.nick_name.clone().unwrap_or(String::from(""));
//     let uid = user::create_user(&conn, nick_name, parameters).await?;
//     #[derive(Serialize)]
//     struct CreateResponse {
//         uid: i32,
//     }
//
//     let resp = CreateResponse { uid };
//     Ok(HttpResponse::Ok().body(NormalResponse::new(resp).to_string()))
// }

// #[post("/user/{uid}/authentication")]
// pub async fn bind_authentication(
//     pool: web::Data<PostgresPool>,
//     form: web::Form<AuthStru>,
//     req: web::HttpRequest,
// ) -> Result<HttpResponse> {
//     let conn = pool.get()?;
//     let parameters: AuthStru = form.into_inner();
//
//     // 参数不存在或不是数字，均返回参数错误 (BadParameter)
//     let uid = req.match_info().get("uid").ok_or(ServerError::from(UserError::BadParameter))?;
//     let uid: i32 = uid.parse().map_err(|_| ServerError::from(UserError::BadParameter))?;
//
//     match parameters {
//         AuthStru {
//             login_type: _LOGIN_BY_WECHAT,
//             wechat_code: Some(wechat_code),
//             ..
//         } => {
//             let wechat_token: WxSession = user::wechat::get_session_by_code(wechat_code.as_str()).await?;
//             user::bind_wechat(&conn, uid, wechat_token.openid).await?;
//         },
//         AuthStru {
//             login_type: _LOGIN_BY_PASSWORD,
//             account: Some(username),
//             credential: Some(password),
//             ..
//         } => {
//             user::bind_password(&conn, uid, username, password).await?;
//         },
//         _ => {
//             return Err(ServerError::from(UserError::BadParameter));
//         }
//     }
//     #[derive(Serialize)]
//     struct EmptyReponse;
//     Ok(HttpResponse::Ok().body(NormalResponse::new(EmptyReponse).to_string()))
// }
