//! This module includes interfaces about go-school-checking.
use crate::error::{Result, ServerError};
use crate::models::checking::{Approval, CheckingError};
use crate::models::user::Person;
use crate::models::{CommonError, PageView};
use crate::services::{EmptyReponse, JwtToken, NormalResponse};
use actix_web::{delete, get, post, web, HttpResponse};
use serde::Deserialize;
use sqlx::postgres::PgPool;

#[derive(Debug, Deserialize)]
pub struct QueryString {
    pub q: Option<String>,
}

#[get("/checking")]
pub async fn list_approvals(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    page: web::Query<PageView>,
    query: web::Query<QueryString>,
) -> Result<HttpResponse> {
    if !token.unwrap().is_admin {
        return Err(CommonError::Forbidden.into());
    }
    let approvement = if let Some(q) = &query.q {
        Approval::search(&pool, q, page.count(50)).await?
    } else {
        Approval::list(&pool, &None, &page).await?
    };
    Ok(HttpResponse::Ok().json(NormalResponse::new(approvement)))
}

#[get("/checking/{uid}")]
pub async fn query_detail(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    uid: web::Path<i32>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let uid = uid.into_inner();

    if uid != token.uid && !token.is_admin {
        return Err(CommonError::Forbidden.into());
    }
    match Approval::query_by_uid(&pool, uid).await {
        Ok(approaval) => Ok(HttpResponse::Ok().json(&NormalResponse::new(approaval))),
        Err(e) => {
            if e == ServerError::new(CheckingError::NoSuchRecord) {
                let identity_result = Person::get_identity(&pool, uid).await?;
                if identity_result.is_none() {
                    return Err(CheckingError::IdentityNeeded.into());
                }
                return Err(CheckingError::NoSuchRecord.into());
            }
            Err(e)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApprovalPost {
    #[serde(rename = "studentId")]
    pub student_id: String,
    pub name: String,
    pub college: String,
    pub major: Option<String>,
}

#[post("/checking")]
pub async fn add_approval(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    data: web::Form<ApprovalPost>,
) -> Result<HttpResponse> {
    if !token.unwrap().is_admin {
        return Err(CommonError::Forbidden.into());
    }
    let submitted = data.into_inner();
    let mut approval = Approval::default();
    approval.student_id = submitted.student_id;
    approval.name = submitted.name;
    approval.college = submitted.college;
    approval.major = submitted.major;
    approval.submit(&pool).await?;

    Ok(HttpResponse::Ok().json(&NormalResponse::new(approval)))
}

#[delete("/checking/{id}")]
pub async fn delete_approval(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse> {
    if !token.unwrap().is_admin {
        return Err(CommonError::Forbidden.into());
    }

    Approval::new(id.into_inner()).delete(&pool).await?;
    Ok(HttpResponse::Ok().json(&EmptyReponse::default()))
}
