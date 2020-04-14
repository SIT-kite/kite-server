use actix_web::{Error, HttpResponse, post, ResponseError, web};
use deadpool_postgres::{ClientWrapper, Pool};
use serde::{Deserialize, Serialize};

use crate::error::ServerError;
use crate::server::middlewares::auth::JwtClaims;
use crate::user;
use crate::user::NormalResponse;
use crate::user::wechat::WxSession;

#[derive(Debug, Deserialize)]
pub struct SessionStru {
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
    form: web::Form<SessionStru>
) -> Result<HttpResponse, ServerError> {
    let conn = pool.get().await?;
    let parameters: SessionStru = form.into_inner();
    let uid;

    // 对应不同登录方式，验证登录凭据，获取用户ID（uid）
    match parameters {
        // 用户名密码方式登录
        SessionStru {
            login_type: LOGIN_USERNAME,
            account: Some(username),
            credential: Some(password),
            ..
        } => {
            uid = user::actions::login(&conn, username, password).await?;
        }
        // 微信方式登录
        SessionStru {
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
            return Ok(HttpResponse::BadRequest().finish());
        }
    }
    // 生成 JWT 字符串并返回
    let claim = JwtClaims { uid };
    let claim_string = serde_json::to_string(&claim).unwrap_or(String::from(""));
    #[derive(Serialize)]
    struct LoginResponse {
        token: String,
    }
    let resp = serde_json::to_string(&LoginResponse { token: claim_string }).unwrap();
    Ok(HttpResponse::Ok().body(NormalResponse::new(resp).to_string()))
}
