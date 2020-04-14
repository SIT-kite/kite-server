use actix_web::{Error, HttpResponse, post, ResponseError, web};
use deadpool_postgres::Pool;
use serde::Deserialize;

use crate::server::error::ServerError;
use crate::user;
use crate::user::models::*;

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
    let mut auth: Verification = Verification::new(form.login_type);

    match form.login_type {
        LOGIN_USERNAME => {
            auth.account = form.account.clone().unwrap().clone();
            auth.credential = form.credential.clone();
        }
        LOGIN_WECHAT => {
            auth.account = form.wechat_code.clone().unwrap().clone();
        }
        _ => {
            return Ok(HttpResponse::BadRequest().finish());
        }
    }


    // Ok(HttpResponse::Ok().body(uid.to_string()))
    Ok(HttpResponse::Ok().body(""))
}
