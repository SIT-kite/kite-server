//! This module includes interfaces about go-school-checking.
use crate::error::{ApiError, Result};
use crate::models::checking::{Administrator, AdministratorManager, CheckingError, StudentStatus};
use crate::models::{CommonError, PageView};
use crate::services::{response::ApiResponse, JwtToken};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::postgres::PgPool;

#[derive(Debug, Deserialize)]
pub struct QueryString {
    pub q: Option<String>,
}

#[get("/checking/student")]
pub async fn list_student_status(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    page: web::Query<PageView>,
    query: web::Query<QueryString>,
) -> Result<HttpResponse> {
    let current_admin = AdministratorManager::get_by_uid(&pool, token.unwrap().uid).await?;
    if let Some(admin) = current_admin {
        let student_status = if let Some(q) = &query.q {
            StudentStatus::search(&pool, q, page.count(50)).await?
        } else {
            StudentStatus::list(&pool, &None, &page).await?
        };
        // Select.
        let student_status: Vec<StudentStatus> = student_status
            .into_iter()
            .filter(|x| admin.role == 3 || x.college == admin.department)
            .collect();
        return Ok(HttpResponse::Ok().json(ApiResponse::normal(student_status)));
    }
    Err(CommonError::Forbidden.into())
}

#[derive(Debug, Deserialize)]
pub struct StudentCredential {
    pub secret: Option<String>,
}

#[get("/checking/student/{student_id}")]
pub async fn query_detail(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    student_id: web::Path<String>,
    form: web::Form<StudentCredential>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let current_admin = AdministratorManager::get_by_uid(&pool, token.uid).await?;
    // Select student status.
    let student = StudentStatus::query(&pool, &student_id.into_inner()).await?;
    // If visitor is administrator, or student himself, return status. Or, the status code (forbidden).
    if let Some(secret) = &form.secret {
        if student.identity_number.ends_with(secret) {
            return Ok(HttpResponse::Ok().json(&ApiResponse::normal(&student)));
        }
        return Err(ApiError::new(CheckingError::CheckIdentity));
    } else {
        if current_admin.is_some() {
            return Ok(HttpResponse::Ok().json(&ApiResponse::normal(&student)));
        }
    }
    Err(CommonError::Forbidden.into())
}

#[derive(Debug, Deserialize)]
pub struct ApprovalPost {
    #[serde(rename = "studentId")]
    pub student_id: String,
    pub name: String,
    pub college: String,
    pub major: Option<String>,
    #[serde(rename = "identityNumber")]
    pub identity_number: String,
    #[serde(rename = "approvalStatus")]
    pub approval_status: Option<bool>,
}

#[post("/checking/student")]
pub async fn add_approval(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    data: web::Form<ApprovalPost>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let submitted = data.into_inner();
    let current_admin = AdministratorManager::get_by_uid(&pool, token.uid).await?;
    // Return forbidden if the user is not belong to administrators.
    if let Some(admin) = current_admin {
        // Check admin privilege.
        if !(admin.role == 3 || admin.department == submitted.college) {
            return Err(CheckingError::DismatchCollege.into());
        }
        if let Ok(_) = StudentStatus::query(&pool, &submitted.student_id).await {
            return Err(CheckingError::StudentExisted.into());
        }
        // Construct student info.
        let mut approval = StudentStatus::default();
        approval.student_id = submitted.student_id;
        approval.name = submitted.name;
        approval.college = submitted.college;
        approval.major = submitted.major;
        approval.identity_number = submitted.identity_number;
        if submitted.approval_status.unwrap_or(false) {
            approval.audit_admin = Some(admin.job_id);
            approval.audit_time = Some(Utc::now().naive_local());
        }
        approval.submit(&pool).await?;

        return Ok(HttpResponse::Ok().json(&ApiResponse::normal(approval)));
    }
    Err(CommonError::Forbidden.into())
}

#[derive(Debug, Deserialize)]
pub struct UpdateStudent {
    #[serde(rename = "approvalStatus")]
    pub approval_status: bool,
}

#[patch("/checking/student/{student_id}")]
pub async fn change_approval(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    student_id: web::Path<String>,
    data: web::Form<UpdateStudent>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let submitted = data.into_inner();
    let current_admin = AdministratorManager::get_by_uid(&pool, token.uid).await?;
    // Return forbidden if the user is not belong to administrators.
    if let Some(admin) = current_admin {
        if let Ok(mut s) = StudentStatus::query(&pool, &student_id).await {
            // Check admin privilege.
            if !(admin.role == 3 || admin.department == s.college) {
                return Err(CheckingError::DismatchCollege.into());
            }
            // Construct student info.
            if submitted.approval_status {
                s.audit_admin = Some(admin.job_id);
                s.audit_time = Some(Utc::now().naive_local());
            } else {
                s.audit_admin = None;
                s.audit_time = None;
            }
            s.update(&pool).await?;

            return Ok(HttpResponse::Ok().json(&ApiResponse::normal(s)));
        }
        return Err(CheckingError::NoSuchStudent.into());
    }
    Err(CommonError::Forbidden.into())
}

#[delete("/checking/student/{student_id}")]
pub async fn delete_approval(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    student_id: web::Path<String>,
) -> Result<HttpResponse> {
    if !token.unwrap().is_admin {
        return Err(CommonError::Forbidden.into());
    }
    StudentStatus::delete(&pool, &student_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[get("/checking/admin")]
pub async fn list_admin(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let current_admin = AdministratorManager::get_by_uid(&pool, token.uid).await?;
    if let Some(admin) = current_admin {
        let admins: Vec<Administrator> = AdministratorManager::list(&pool, &page)
            .await?
            .into_iter()
            .filter(|x| x.role <= admin.role)
            .collect();
        return Ok(HttpResponse::Ok().json(&ApiResponse::normal(admins)));
    }
    Err(CommonError::Forbidden.into())
}

#[delete("/checking/admin/{job_id}")]
pub async fn delete_admin(
    token: Option<JwtToken>,
    pool: web::Data<PgPool>,
    job_id: web::Path<String>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let current_admin = AdministratorManager::get_by_uid(&pool, token.uid)
        .await?
        .ok_or(ApiError::new(CommonError::Forbidden))?;
    let target_admin = AdministratorManager::get(&pool, &job_id.into_inner())
        .await?
        .ok_or(ApiError::new(CheckingError::NoSuchAdmin))?;
    /* Is there someone going to delete his account? */

    if current_admin.role < target_admin.role
        || (current_admin.role != 3 && current_admin.department != target_admin.department)
    {
        return Err(CheckingError::DismatchCollege.into());
    }
    AdministratorManager::delete(&pool, &target_admin.job_id).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}
