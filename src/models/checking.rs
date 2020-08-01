use crate::error::{ApiError, Result};
use crate::models::PageView;
use chrono::NaiveDateTime;
use num_traits::ToPrimitive;
use serde::Serialize;
use sqlx::{postgres::PgQueryAs, PgPool};
use thiserror::Error;

#[derive(Debug, Error, ToPrimitive)]
pub enum CheckingError {
    #[error("该学号不存在")]
    NoSuchStudent = 1001,
    #[error("请检查信息是否填写错误")]
    CheckIdentity = 1002,
    #[error("需要先实名认证")]
    IdentityNeeded = 1003,
    #[error("该工号已存在")]
    AdminExisted = 1004,
    #[error("不能跨学院操作")]
    DismatchCollege = 1005,
    #[error("找不到要删除的管理员")]
    NoSuchAdmin = 1006,
    #[error("该学号已存在")]
    StudentExisted = 1007,
}

impl Into<ApiError> for CheckingError {
    fn into(self) -> ApiError {
        ApiError {
            code: self.to_u16().unwrap(), // Error code
            inner_msg: None,
            error_msg: Some(self.to_string()), // Error message
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StudentStatus {
    /// Student ID
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// Student account uid if registered.
    pub uid: Option<i32>,
    /// Real name
    pub name: String,
    /// Approved time
    #[serde(rename = "approvedTime")]
    pub audit_time: Option<NaiveDateTime>,
    #[serde(rename = "approvedAdmin")]
    pub audit_admin: Option<String>,
    /* Belows are some personal information */
    /// Student college
    pub college: String,
    /// Student major, optional
    pub major: Option<String>,
    /// Identity number.
    pub identity_number: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Administrator {
    /// Job number, usually a 4 bit number like 1024 :D
    pub job_id: String,
    /// Admin name
    pub name: String,
    /// Department
    pub department: String,
    /// UID of administrator
    pub uid: i32,
    /// Role
    pub role: i16,
}

impl StudentStatus {
    /// Save to database.
    pub async fn submit(&self, client: &PgPool) -> Result<()> {
        let _ = sqlx::query(
            "INSERT INTO checking.approvals 
                    (student_id, name, audit_time, audit_admin, college, major, identity_number)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (student_id)
                DO UPDATE SET audit_time = $3, audit_admin = $4",
        )
        .bind(&self.student_id)
        .bind(&self.name)
        .bind(&self.audit_time)
        .bind(&self.audit_admin)
        .bind(&self.college)
        .bind(&self.major)
        .bind(&self.identity_number)
        .execute(client)
        .await?;
        Ok(())
    }

    /// Get personal information and whether he is approved in BY student-id.
    pub async fn get(client: &PgPool, student_id: &String) -> Result<Self> {
        let approval_record: Option<StudentStatus> = sqlx::query_as(
            "SELECT student_id, uid, name, audit_time, audit_admin, college, major, identity_number
                FROM checking.approval_view
                WHERE student_id = $1 LIMIT 1",
        )
        .bind(student_id)
        .fetch_optional(client)
        .await?;
        approval_record.ok_or(CheckingError::NoSuchStudent.into())
    }

    /// Delete approve record.
    pub async fn delete(client: &PgPool, student_id: &String) -> Result<()> {
        let _ = sqlx::query("DELETE FROM checking.approvals WHERE student_id = $1")
            .bind(student_id)
            .execute(client)
            .await?;
        Ok(())
    }

    /// Get Approve List
    pub async fn list(client: &PgPool, college: &Option<String>, page: &PageView) -> Result<Vec<Self>> {
        let approve_list = sqlx::query_as(
            "SELECT student_id, uid, name, audit_time, audit_admin, college, major, identity_number
                FROM checking.approval_view
                WHERE college LIKE $1 ORDER BY audit_time DESC 
                OFFSET $2 LIMIT $3",
        )
        .bind(if let Some(college) = college {
            // Note: This field may lead postgres scanning full table.
            format!("%{}%", college)
        } else {
            "%".to_string()
        })
        .bind(page.offset(50) as i32)
        .bind(page.count(50) as i32)
        .fetch_all(client)
        .await?;
        Ok(approve_list)
    }

    /// Search student name
    pub async fn search(client: &PgPool, query_string: &String, count: u16) -> Result<Vec<Self>> {
        let result: Vec<Self> = sqlx::query_as(
            "SELECT student_id, uid, name, audit_time, audit_admin, college, major, identity_number
                FROM checking.approval_view
                WHERE name LIKE $1 ORDER BY audit_time DESC LIMIT $2",
        )
        .bind(format!("%{}%", query_string))
        .bind(count as i32)
        .fetch_all(client)
        .await?;
        Ok(result)
    }
}

/// Administrator manager.
pub struct AdministratorManager {}

impl AdministratorManager {
    pub async fn list(pool: &PgPool, page: &PageView) -> Result<Vec<Administrator>> {
        let administrators: Vec<Administrator> = sqlx::query_as(
            "SELECT job_id, name, department, uid, role
            FROM checking.administrators
            ORDER BY department, role DESC
            LIMIT $1 OFFSET $2",
        )
        .bind(page.count(50) as i16)
        .bind(page.offset(50) as i16)
        .fetch_all(pool)
        .await?;
        Ok(administrators)
    }

    pub async fn get(pool: &PgPool, job_id: &String) -> Result<Option<Administrator>> {
        let administrator: Option<Administrator> = sqlx::query_as(
            "SELECT job_id, name, department, uid, role
            FROM checking.administrators
            WHERE job_id = $1
            LIMIT 1",
        )
        .bind(job_id)
        .fetch_optional(pool)
        .await?;
        Ok(administrator)
    }

    pub async fn get_by_uid(pool: &PgPool, uid: i32) -> Result<Option<Administrator>> {
        let result: Option<Administrator> = sqlx::query_as(
            "SELECT job_id, name, department, uid, role
                FROM checking.administrators 
                WHERE uid = $1 LIMIT 1",
        )
        .bind(uid)
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }
    pub async fn add(pool: &PgPool, admin_user: Administrator) -> Result<()> {
        if Self::get(pool, &admin_user.job_id).await?.is_some() {
            return Err(CheckingError::AdminExisted.into());
        }
        let _ = sqlx::query(
            "INSERT INTO checking.administrators (job_id, name, department, uid, role)
                    VALUES ($1, $2, $3, $4)",
        )
        .bind(admin_user.uid)
        .bind(admin_user.name)
        .bind(admin_user.department)
        .bind(admin_user.job_id)
        .bind(admin_user.role)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &PgPool, job_id: &String) -> Result<()> {
        let _ = sqlx::query("DELETE FROM checking.administrators WHERE job_id = $1")
            .bind(job_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl Default for StudentStatus {
    fn default() -> Self {
        StudentStatus {
            student_id: String::from(""),
            uid: None,
            name: String::from(""),
            audit_time: None,
            college: String::from(""),
            major: None,
            audit_admin: None,
            identity_number: String::from(""),
        }
    }
}
