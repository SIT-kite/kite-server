//! This module includes interfaces about freshman queries.
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use actix_web::error::BlockingError;
use chrono::{NaiveDateTime, Utc};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ServerError};
use crate::freshman::{self, FreshmanError};
use crate::server::JwtTokenBox;

use super::super::NormalResponse;

pub type Pool<T> = diesel::r2d2::Pool<ConnectionManager<T>>;
pub type PostgresPool = Pool<PgConnection>;

#[derive(Debug, Deserialize)]
pub struct FreshmanEnvReq {
    pub secret: Option<String>,
}

#[get("/event")]
pub async fn get_environment(
    pool: web::Data<PostgresPool>,
    token: JwtTokenBox,
    path: web::Path<String>,
    form: web::Form<FreshmanEnvReq>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    let parameters: FreshmanEnvReq = form.into_inner();
    let uid = token.value.unwrap().uid;

    let environment = web::block(move || freshman::get_env_by_uid(&conn, uid)).await;
    match environment {
        Ok(env) =>
        // If current has logon before.
            {
                return Ok(HttpResponse::Ok().json(&NormalResponse::new(env)))
            }
        Err(e) => {
            // When error occurred while get_env_by_uid()
            if let BlockingError::Error(srv_err) = e {
                // Current user have not bound yet.
                if srv_err == ServerError::new(FreshmanError::NoSuchAccount) {
                    let account = path.into_inner();
                    if let Some(secret) = parameters.secret {
                        let conn = pool.get()?;
                        // Try to bind user to his student id
                        web::block(move || freshman::bind_account(&conn, uid, &account, &secret))
                            .await;
                    }
                }
                return Err(srv_err);
            }
        }
    }
    // TODO.
    Err(ServerError::new(FreshmanError::NoSuchAccount))
}
