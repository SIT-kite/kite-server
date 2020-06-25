//! This module includes interfaces about freshman queries.
use actix_web::error::BlockingError;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use super::super::{EmptyReponse, NormalResponse};
use crate::error::{Result, ServerError};
use crate::freshman::{self, FreshmanBasic, FreshmanError, NewMate, PeopleFamiliar};
use crate::server::JwtToken;

#[derive(Debug, Deserialize)]
pub struct FreshmanEnvReq {
    pub secret: Option<String>,
}

#[get("/freshman/{account}")]
pub async fn get_basic_info(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    form: web::Form<FreshmanEnvReq>,
) -> Result<HttpResponse> {
    /* Auth middleware may be sure that all of users are authenticated. */
    // If someone didn't login before.
    if None == token {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.unwrap();
    let parameters: FreshmanEnvReq = form.into_inner();

    let uid = token.uid;
    let account = &path.into_inner();
    let secret = parameters.secret;

    #[derive(Serialize)]
    struct BasicInfo {
        pub me: FreshmanBasic,
        #[serde(rename(serialize = "sameNameCount"))]
        pub same_name_count: i64,
    }
    match secret {
        Some(secret) => {
            if ! freshman::is_uid_bound_with(&pool, uid, &account).await? {
                // When uid is not bound, bind uid to student.
                freshman::bind_account(&pool, uid, account, &secret).await?;
            }
            let student = freshman::get_basic_info_by_account(&pool, account, &secret).await?;
            let self_basic = BasicInfo {
                me: student,
                same_name_count: freshman::get_count_of_same_name(&pool, uid).await? - 1,
            };
            return Ok(HttpResponse::Ok().json(NormalResponse::new(self_basic)));
        }
        None => return Err(ServerError::new(FreshmanError::SecretNeeded)),
    }
}

#[derive(Deserialize)]
pub struct UpdateInfo {
    pub contact: Option<String>,
    pub visible: Option<bool>,
    pub last_seen: Option<bool>,
}

#[put("/freshman/{account}")]
pub async fn update_account(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    form: web::Form<UpdateInfo>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.unwrap();
    let uid = token.uid;

    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if !freshman::is_uid_bound_with(&pool, uid, &account).await? {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }

    // Set visibility.
    let form = form.into_inner();
    if let Some(visible) = form.visible {
        freshman::set_visibility(&pool, uid, visible).await?;
    }
    // Set contact information.
    if let Some(contact) = form.contact {
        let contact_json: serde_json::Value = serde_json::from_str(contact.as_str())?;
        freshman::update_contact_by_uid(&pool, uid, &contact_json).await?;
    }
    // Update last seen.
    if let Some(last_seen) = form.last_seen {
        freshman::update_last_seen(&pool, uid).await?;
    }
    Ok(HttpResponse::Ok().json(EmptyReponse::default()))
}

#[get("/freshman/{account}/roommate")]
pub async fn get_roommate(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.unwrap();
    let uid = token.uid;

    #[derive(Serialize)]
    struct Resp {
        pub roommates: Vec<NewMate>,
    }
    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if freshman::is_uid_bound_with(&pool, uid, &account).await? {
        let resp = Resp {
            roommates: freshman::get_roommates_by_uid(&pool, uid).await?,
        };
        Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
    } else {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }
}

#[get("/freshman/{account}/familiar")]
pub async fn get_people_familiar(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.unwrap();
    let uid = token.uid;

    #[derive(Serialize)]
    struct Resp {
        pub fellows: Vec<PeopleFamiliar>,
    }
    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if !freshman::is_uid_bound_with(&pool, uid, &account).await? {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }
    let resp = Resp {
        fellows: freshman::get_people_familiar_by_uid(&pool, uid).await?,
    };
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}

#[get("/freshman/{account}/classmate")]
pub async fn get_classmate(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.unwrap();
    let uid = token.uid;

    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if !freshman::is_uid_bound_with(&pool, uid, &account).await? {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }
    #[derive(Serialize)]
    struct Resp {
        pub classmates: Vec<NewMate>,
    }
    let resp = Resp {
        classmates: freshman::get_classmates_by_uid(&pool, uid).await?,
    };
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}
