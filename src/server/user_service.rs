use serde::Deserialize;
use actix_web::{web, HttpResponse, Error, post, ResponseError};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use crate::user::models::*;
use crate::user;
use crate::server::error::ServerError;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;


#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct SessionStru {
    // loginType. Defined in user/models.rs, value can be either LOGIN_WECHAT
    // or LOGIN_USERNAME
    loginType: i32,
    // The code that provided by wechat wx.login()
    wxCode: Option<String>,
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
    let mut auth: Verification = Verification::new(form.loginType);

    match form.loginType {
        LOGIN_USERNAME => {
            auth.account = form.account.clone().unwrap().clone();
            auth.credential = form.credential.clone();
        }
        LOGIN_WECHAT => {
            auth.account = form.wxCode.clone().unwrap().clone();
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
