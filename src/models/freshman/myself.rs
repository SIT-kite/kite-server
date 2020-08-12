use super::{FreshmanBasic, FreshmanError};
use crate::error::{ApiError, Result};
use sqlx::postgres::{PgPool, PgQueryAs};

impl FreshmanBasic {
    pub async fn get_contact(&self, pool: &PgPool) -> Result<serde_json::Value> {
        let result: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT contact FROM freshman.students WHERE student_id = $1 LIMIT 1")
                .bind(&self.student_id)
                .fetch_optional(pool)
                .await?;
        result
            .map(|x| x.0)
            .ok_or(ApiError::new(FreshmanError::NoSuchAccount))
    }

    pub async fn set_contact(&self, pool: &PgPool, contact: serde_json::Value) -> Result<()> {
        sqlx::query("UPDATE freshman.students SET contact = $1 WHERE student_id = $2")
            .bind(&contact)
            .bind(&self.student_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn set_visibility(&self, pool: &PgPool, visible: bool) -> Result<()> {
        sqlx::query("UPDATE freshman.students SET visible = $1 WHERE student_id = $2")
            .bind(visible)
            .bind(&self.student_id)
            .execute(pool)
            .await?;
        Ok(())
    }
    // End of impl FreshmanBasic
}

pub struct FreshmanManager<'a> {
    pool: &'a PgPool,
}

impl<'a> FreshmanManager<'a> {
    /// Create a new freshman manager.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Query student basic.
    ///
    /// Query string can be name, student id, or ticket number.
    pub async fn query(&self, query_string: &str, secret: &str) -> Result<FreshmanBasic> {
        let results = self.query_batch(query_string).await?;

        for each_freshman in results {
            if each_freshman.secret == secret {
                return Ok(each_freshman);
            }
        }
        Err(ApiError::new(FreshmanError::NoSuchAccount))
    }

    /// Query student basic without secret. Provided for administrators or other management approaches.
    pub async fn query_batch(&self, query_string: &str) -> Result<Vec<FreshmanBasic>> {
        let student_basic: Vec<FreshmanBasic> = sqlx::query_as(
            "SELECT
                    uid, student_id, college, major, campus, building, room, bed, secret,
                    counselor_name, counselor_tel, visible
                FROM freshman.students
                WHERE name = $1 or student_id = $1 or ticket = $1",
        )
        .bind(query_string)
        .fetch_all(self.pool)
        .await?;
        Ok(student_basic)
    }

    /// Bind student id with uid.
    pub async fn bind(&self, student_id: &str, uid: Option<i32>) -> Result<()> {
        sqlx::query("UPDATE freshman.students SET uid = $1 WHERE student_id = $2")
            .bind(uid)
            .bind(student_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_bound(&self, uid: i32) -> Result<bool> {
        let r: Option<(bool,)> =
            sqlx::query_as("SELECT TRUE FROM freshman.students WHERE uid = $1 LIMIT 1")
                .bind(uid)
                .fetch_optional(self.pool)
                .await?;
        Ok(r.is_some())
    }
}
