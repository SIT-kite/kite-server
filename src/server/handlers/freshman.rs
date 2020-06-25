//! This module includes interfaces about freshman queries.
use actix_web::error::BlockingError;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use super::super::{EmptyReponse, NormalResponse};
use crate::error::{Result, ServerError};
use crate::freshman::{self, FreshmanBasic, FreshmanError, NewMate, PeopleFamiliar};
use crate::server::JwtTokenBox;

#[derive(Debug, Deserialize)]
pub struct FreshmanEnvReq {
    pub secret: Option<String>,
}

#[get("/freshman/{account}")]
pub async fn get_basic_info(
    pool: web::Data<PgPool>,
    token: JwtTokenBox,
    path: web::Path<String>,
    form: web::Form<FreshmanEnvReq>,
) -> Result<HttpResponse> {
    /* Auth middleware may be sure that all of users are authenticated. */
    // If someone didn't login before.
    if None == token.value {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.value.unwrap();
    let uid = token.uid;
    let account = &path.into_inner();

    // If the uid is bound to one student, return result immediately.
    if token.is_admin || freshman::is_uid_bound_with(&pool, uid, &account).await? {
        if let student = freshman::get_basic_info(&pool, uid).await? {
            return Ok(HttpResponse::Ok().json(student));
        }
    }

    /* When uid is not bound, bind uid to student. */
    let parameters: FreshmanEnvReq = form.into_inner();
    let secret = parameters.secret;
    match secret {
        Some(secret) => {
            freshman::bind_account(&pool, uid, account, &secret).await?;

            let student = freshman::get_basic_info(&pool, uid).await?;
            return Ok((HttpResponse::Ok().json(student)));
        }
        None => return Err(ServerError::new(FreshmanError::SecretNeeded)),
    }
}

#[derive(Deserialize)]
pub struct UpdateInfo {
    pub contact: Option<String>,
    pub visible: Option<bool>,
}

#[put("/freshman/{account}")]
pub async fn update_account(
    pool: web::Data<PgPool>,
    token: JwtTokenBox,
    path: web::Path<String>,
    form: web::Form<UpdateInfo>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token.value {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.value.unwrap();
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
    Ok(HttpResponse::Ok().json(EmptyReponse))
}

#[get("/freshman/{account}/roommate")]
pub async fn get_roommate(
    pool: web::Data<PgPool>,
    token: JwtTokenBox,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token.value {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.value.unwrap();
    let uid = token.uid;

    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if !token.is_admin && !freshman::is_uid_bound_with(&pool, uid, &account).await? {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }
    #[derive(Serialize)]
    struct Resp {
        pub roommates: Vec<NewMate>,
    }
    let resp = Resp {
        roommates: freshman::get_roommates(&pool, uid).await?,
    };
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}

#[get("/freshman/{account}/familiar")]
pub async fn get_people_familiar(
    pool: web::Data<PgPool>,
    token: JwtTokenBox,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token.value {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.value.unwrap();
    let uid = token.uid;

    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if !token.is_admin && !freshman::is_uid_bound_with(&pool, uid, &account).await? {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }
    #[derive(Serialize)]
    struct Resp {
        pub fellows: Vec<PeopleFamiliar>,
    }
    let resp = Resp {
        fellows: freshman::get_people_familiar(&pool, uid).await?,
    };
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}

#[get("/freshman/{account}/classmate")]
pub async fn get_classmate(
    pool: web::Data<PgPool>,
    token: JwtTokenBox,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // If someone didn't login before.
    if None == token.value {
        // Return error code : Forbidden
        return Err(ServerError::new(FreshmanError::Forbidden));
    }
    let token = token.value.unwrap();
    let uid = token.uid;

    // Check if the account is bound to this uid.
    let account = path.into_inner();
    if !token.is_admin && !freshman::is_uid_bound_with(&pool, uid, &account).await? {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }
    #[derive(Serialize)]
    struct Resp {
        pub fellows: Vec<NewMate>,
    }
    let resp = Resp {
        fellows: freshman::get_classmates(&pool, uid).await?,
    };
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}
