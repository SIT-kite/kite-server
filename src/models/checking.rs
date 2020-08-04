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

#[derive(Default, Serialize, sqlx::FromRow)]
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

#[derive(Serialize, sqlx::FromRow)]
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

pub struct StudentManager<'a> {
    pool: &'a PgPool,
}

/// Administrator manager.
pub struct AdministratorManager<'a> {
    pool: &'a PgPool,
}

impl<'a> StudentManager<'a> {
    /// Create a new student manager object.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Count
    pub async fn count(&self, college: &String) -> Result<i64> {
        let count: Option<(i64,)> =
            sqlx::query_as("SELECT COUNT(student_id) FROM checking.students WHERE college LIKE $1")
                .bind(college)
                .fetch_optional(self.pool)
                .await?;
        Ok(count.unwrap().0)
    }

    /// Save to database.
    pub async fn submit(&self, student: &StudentStatus) -> Result<()> {
        let _ = sqlx::query(
            "INSERT INTO checking.students 
                    (student_id, name, audit_tim , audit_admin, college, major, identity_number)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (student_id)
                DO UPDATE SET audit_time = $3, audit_admin = $4",
        )
        .bind(&student.student_id)
        .bind(&student.name)
        .bind(&student.audit_time)
        .bind(&student.audit_admin)
        .bind(&student.college)
        .bind(&student.major)
        .bind(&student.identity_number)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// Get personal information and whether he is approved in BY student-id.
    pub async fn get(&self, student_id: &String) -> Result<StudentStatus> {
        let approval_record: Option<StudentStatus> = sqlx::query_as(
            "SELECT student_id, uid, name, audit_time, audit_admin, college, major, identity_number
                FROM checking.approval_view
                WHERE student_id = $1 LIMIT 1",
        )
        .bind(student_id)
        .fetch_optional(self.pool)
        .await?;
        approval_record.ok_or(CheckingError::NoSuchStudent.into())
    }

    /// Delete approve record.
    pub async fn delete(&self, student_id: &String) -> Result<()> {
        sqlx::query("DELETE FROM checking.students WHERE student_id = $1")
            .bind(student_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Get Approve List
    pub async fn list(
        &self,
        query_string: Option<String>,
        college_domain: &String,
        page: &PageView,
    ) -> Result<Vec<StudentStatus>> {
        let query_string = query_string
            .map(|x| format!("%{}%", x))
            .unwrap_or("%".to_string());

        let approve_list = sqlx::query_as(
            "SELECT student_id, uid, name, audit_time, audit_admin, college, major, identity_number
                FROM checking.approval_view
                WHERE (college LIKE $1 OR name LIKE $1 OR student_id LIKE $1) AND college LIKE $2
                ORDER BY student_id
                OFFSET $3 LIMIT $4",
        )
        .bind(query_string)
        .bind(college_domain)
        .bind(page.offset(50) as i32)
        .bind(page.count(50) as i32)
        .fetch_all(self.pool)
        .await?;
        Ok(approve_list)
    }
}

impl<'a> AdministratorManager<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, page: &PageView) -> Result<Vec<Administrator>> {
        let administrators: Vec<Administrator> = sqlx::query_as(
            "SELECT job_id, name, department, uid, role
            FROM checking.administrators
            ORDER BY department, role DESC
            LIMIT $1 OFFSET $2",
        )
        .bind(page.count(50) as i16)
        .bind(page.offset(50) as i16)
        .fetch_all(self.pool)
        .await?;
        Ok(administrators)
    }

    pub async fn get(&self, job_id: &String) -> Result<Option<Administrator>> {
        let administrator: Option<Administrator> = sqlx::query_as(
            "SELECT job_id, name, department, uid, role
            FROM checking.administrators
            WHERE job_id = $1
            LIMIT 1",
        )
        .bind(job_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(administrator)
    }

    pub async fn get_by_uid(&self, uid: i32) -> Result<Option<Administrator>> {
        let result: Option<Administrator> = sqlx::query_as(
            "SELECT job_id, name, department, uid, role
                FROM checking.administrators 
                WHERE uid = $1 LIMIT 1",
        )
        .bind(uid)
        .fetch_optional(self.pool)
        .await?;
        Ok(result)
    }
    pub async fn add(&self, admin_user: Administrator) -> Result<()> {
        if self.get(&admin_user.job_id).await?.is_some() {
            return Err(CheckingError::AdminExisted.into());
        }
        sqlx::query(
            "INSERT INTO checking.administrators (job_id, name, department, uid, role)
                    VALUES ($1, $2, $3, $4)",
        )
        .bind(admin_user.uid)
        .bind(admin_user.name)
        .bind(admin_user.department)
        .bind(admin_user.job_id)
        .bind(admin_user.role)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, job_id: &String) -> Result<()> {
        let _ = sqlx::query("DELETE FROM checking.administrators WHERE job_id = $1")
            .bind(job_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
