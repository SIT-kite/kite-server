//! This module includes interfaces about go-school-checking.
use crate::error::{ApiError, Result};
use crate::models::checking::{
    Administrator, AdministratorManager, CheckingError, StudentManager, StudentStatus,
};
use crate::models::{CommonError, PageView};
use crate::services::{response::ApiResponse, AppState, JwtToken};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct QueryString {
    pub q: Option<String>,
}

#[get("/checking/student")]
pub async fn list_student_status(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    page: web::Query<PageView>,
    query: web::Query<QueryString>,
) -> Result<HttpResponse> {
    let query_string = query.into_inner().q;

    // To check whether query string is empty or not. If it's not empty...
    let admin_manager = AdministratorManager::new(&app.pool);
    let student_manager = StudentManager::new(&app.pool);

    // Get role (as privilege) and college of current admin account.
    let current_admin = admin_manager.get_by_uid(token.unwrap().uid).await?;
    if let Some(admin) = current_admin {
        // Top admin can list all of students, while college admin could only see which he's belong to.
        let college_domain = if admin.role == 3 {
            "%".to_string()
        } else {
            admin.department
        };
        // Note:
        // If query string is None, return all results.
        // If query string exists, while it's empty, return all results.
        // In both situation, the count is all student count (may be under a certain college).
        // Otherwise, return selected students and selected count.
        let students = student_manager
            .list(query_string.clone(), &college_domain, &page)
            .await?;
        let mut count = students.len() as u32;
        if query_string.unwrap_or_default().is_empty() {
            count = student_manager.count(&college_domain).await? as u32;
        }

        #[derive(Serialize)]
        struct Response {
            count: u32,
            students: Vec<StudentStatus>,
        }
        Ok(HttpResponse::Ok().json(ApiResponse::normal(&Response { count, students })))
    } else {
        return Err(CommonError::Forbidden.into());
    }
}

#[derive(Debug, Deserialize)]
pub struct StudentCredential {
    pub secret: Option<String>,
}

#[get("/checking/student/{student_id}")]
pub async fn query_detail(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    student_id: web::Path<String>,
    form: web::Form<StudentCredential>,
) -> Result<HttpResponse> {
    let token = token.unwrap();

    let admin_manager = AdministratorManager::new(&app.pool);
    let current_admin = admin_manager.get_by_uid(token.uid).await?;
    let student_manager = StudentManager::new(&app.pool);

    // Select student status.
    let student = student_manager.get(&student_id.into_inner()).await?;
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
    app: web::Data<AppState>,
    data: web::Form<ApprovalPost>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let submitted = data.into_inner();
    let admin_manager = AdministratorManager::new(&app.pool);
    let current_admin = admin_manager.get_by_uid(token.uid).await?;

    // Return forbidden if the user is not belong to administrators.
    if let Some(admin) = current_admin {
        // Check admin privilege.
        if !(admin.role == 3 || admin.department == submitted.college) {
            return Err(CheckingError::DismatchCollege.into());
        }
        let student_manager = StudentManager::new(&app.pool);
        if let Ok(_) = student_manager.get(&submitted.student_id).await {
            return Err(CheckingError::StudentExisted.into());
        }
        // Construct student info.
        let mut student = StudentStatus::default();
        student.student_id = submitted.student_id;
        student.name = submitted.name;
        student.college = submitted.college;
        student.major = submitted.major;
        student.identity_number = submitted.identity_number;
        if submitted.approval_status.unwrap_or(false) {
            student.audit_admin = Some(admin.job_id);
            student.audit_time = Some(Utc::now().naive_local());
        }
        student_manager.submit(&student).await?;

        return Ok(HttpResponse::Ok().json(&ApiResponse::normal(student)));
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
    app: web::Data<AppState>,
    student_id: web::Path<String>,
    data: web::Form<UpdateStudent>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let submitted = data.into_inner();
    let admin_manager = AdministratorManager::new(&app.pool);
    let current_admin = admin_manager.get_by_uid(token.uid).await?;
    // Return forbidden if the user is not belong to administrators.
    if let Some(admin) = current_admin {
        let student_manager = StudentManager::new(&app.pool);

        if let Ok(mut s) = student_manager.get(&student_id).await {
            // Check admin privilege.
            if !(admin.role == 3 || admin.department == s.college) {
                return Err(CheckingError::DismatchCollege.into());
            }
            // Construct student info.
            if submitted.approval_status {
                s.audit_admin = Some(admin.job_id.clone());
                s.audit_time = Some(Utc::now().naive_local());
            } else {
                s.audit_admin = None;
                s.audit_time = None;
            }
            student_manager.submit(&s).await?;
            // FIX BUG: Return audit_admin with name and job id.
            if submitted.approval_status {
                s.audit_admin = Some(format!("{} （{}）", admin.name, admin.job_id));
            }

            return Ok(HttpResponse::Ok().json(&ApiResponse::normal(s)));
        }
        return Err(CheckingError::NoSuchStudent.into());
    }
    Err(CommonError::Forbidden.into())
}

#[delete("/checking/student/{student_id}")]
pub async fn delete_approval(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    student_id: web::Path<String>,
) -> Result<HttpResponse> {
    let admin_manager = AdministratorManager::new(&app.pool);
    let student_manager = StudentManager::new(&app.pool);

    if admin_manager.get_by_uid(token.unwrap().uid).await?.is_none() {
        return Err(CommonError::Forbidden.into());
    }
    student_manager.delete(&student_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[get("/checking/admin")]
pub async fn list_admin(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let admin_manager = AdministratorManager::new(&app.pool);
    let current_admin = admin_manager.get_by_uid(token.uid).await?;

    if let Some(admin) = current_admin {
        let admins = admin_manager
            .list(&page)
            .await?
            .into_iter()
            .filter(|x| x.role <= admin.role)
            .collect::<Vec<Administrator>>();
        return Ok(HttpResponse::Ok().json(&ApiResponse::normal(admins)));
    }
    Err(CommonError::Forbidden.into())
}

#[delete("/checking/admin/{job_id}")]
pub async fn delete_admin(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    job_id: web::Path<String>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let admin_manager = AdministratorManager::new(&app.pool);

    let current_admin = admin_manager
        .get_by_uid(token.uid)
        .await?
        .ok_or(ApiError::new(CommonError::Forbidden))?;
    let target_admin = admin_manager
        .get(&job_id.into_inner())
        .await?
        .ok_or(ApiError::new(CheckingError::NoSuchAdmin))?;

    /* Is there someone going to delete his account? */
    if current_admin.role < target_admin.role
        || (current_admin.role != 3 && current_admin.department != target_admin.department)
    {
        return Err(CheckingError::DismatchCollege.into());
    }
    admin_manager.delete(&target_admin.job_id).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}
