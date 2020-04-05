use actix_web::{Error, HttpResponse, post, ResponseError, web};
use diesel::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use serde::Deserialize;

use crate::server::error::ServerError;
use crate::user;
use crate::user::models::*;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;


#[allow(non_snake_case)]
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
    pool: web::Data<DbPool>,
    form: web::Form<SessionStru>
) -> Result<HttpResponse, ServerError> {

    let conn = pool.get()?;
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
    // Run database process in a separated thread.
    let uid = web::block(move || user::actions::login(&conn, auth))
        .await
        .map_err(|e| {
            ServerError::Block
        })?;

    Ok(HttpResponse::Ok().body(uid.to_string()))
}
