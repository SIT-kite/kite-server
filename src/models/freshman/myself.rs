use super::FreshmanError;
use crate::error::{ApiError, Result};
use serde::Serialize;
use sqlx::postgres::{PgPool, PgQueryAs};

/// FreshmanEnv
/// Used to express campus, dormitory, counselor and other environment variables
/// for each new student.
/// Note: This structure is used to query only.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct FreshmanBasic {
    pub uid: Option<i32>,
    /// student id.
    #[serde(rename(serialize = "studentId"))]
    pub student_id: String,
    /// Freshman college
    pub college: String,
    /// Freshman major
    pub major: String,
    /// campus of Fengxian or Xuhui.
    pub campus: String,
    /// like "1号楼". For Xuhui has some buildings named like "南1号楼", we use a string.
    pub building: String,
    /// like "101"
    pub room: i32,
    /// like "101-1"
    pub bed: String,
    /// Counselor's name
    #[serde(rename(serialize = "counselorName"))]
    pub counselor_name: String,
    /// Counselor's telephone
    #[serde(rename(serialize = "counselorTel"))]
    pub counselor_tel: String,
    /// Allow people in the same city access one's contact details.
    pub visible: bool,
}

pub async fn update_last_seen(client: &PgPool, uid: i32) -> Result<()> {
    let _: u64 = sqlx::query("UPDATE freshman.students SET last_seen = now() WHERE uid = $1;")
        .bind(uid)
        .execute(client)
        .await?;
    Ok(())
}

pub async fn is_account_bound(client: &PgPool, account: &String, secret: &String) -> Result<bool> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM freshman.students
        WHERE (name = $1 or student_id = $1 or ticket = $1) and secret = $2 and uid is not null",
    )
    .bind(account)
    .bind(secret)
    .fetch_optional(client)
    .await?;
    Ok(row.is_some())
}

pub async fn is_uid_bound_with(client: &PgPool, uid: i32, account: &String) -> Result<bool> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM freshman.students
        WHERE uid = $1 AND (name = $2 or student_id = $2 or ticket = $2)",
    )
    .bind(uid)
    .bind(account)
    .fetch_optional(client)
    .await?;
    Ok(row.is_some())
}

/// Bind account(name, student_id, ticket) to uid.
/// Note: There are two SQL queries in the function, and the first is the checking of whether
/// they are bound. so data synchronization problems may occur.
/// While if the account is not existing, it will return FreshmanError::NoSuchAccount.
/// Normally, it returns a String as student_id.
pub async fn bind_account(
    client: &PgPool,
    uid: i32,
    account: &String,
    secret: &String,
) -> Result<String> {
    let student_id: Option<(String,)> = sqlx::query_as(
        "UPDATE freshman.students SET uid = $1 \
        WHERE (name = $2 or student_id = $2 or ticket = $2) and secret = $3 and uid is null \
        RETURNING student_id",
    )
    .bind(uid)
    .bind(account)
    .bind(secret)
    .fetch_optional(client)
    .await?;

    // If the account is not existing, return FreshmanError::NoSuchAccount,
    // else return student_id.
    match student_id {
        Some(valid_id) => Ok(valid_id.0),
        None => Err(ApiError::new(FreshmanError::NoSuchAccount)),
    }
}

pub async fn get_basic_info_by_account(
    client: &PgPool,
    account: &String,
    secret: &String,
) -> Result<FreshmanBasic> {
    let student_basic: Option<FreshmanBasic> = sqlx::query_as::<_, FreshmanBasic>(
        "SELECT
                uid, student_id, college, major, campus, building, room, bed,
                counselor_name, counselor_tel, visible
            FROM freshman.students
            WHERE (name = $1 or student_id = $1 or ticket = $1) and secret = $2",
    )
    .bind(account)
    .bind(secret)
    .fetch_optional(client)
    .await?;

    match student_basic {
        Some(e) => Ok(e),
        None => Err(ApiError::new(FreshmanError::NoSuchAccount)),
    }
}

pub async fn update_contact_by_uid(
    client: &PgPool,
    uid: i32,
    new_contact: &serde_json::Value,
) -> Result<()> {
    let _: u64 = sqlx::query("UPDATE students SET contact = $1 WHERE uid = $2")
        .bind(new_contact)
        .bind(uid)
        .execute(client)
        .await?;

    Ok(())
}

pub async fn set_visibility(client: &PgPool, uid: i32, visible: bool) -> Result<()> {
    let _: u64 = sqlx::query("UPDATE students SET visibility = $1 WHERE uid = $2")
        .bind(visible)
        .bind(uid)
        .execute(client)
        .await?;

    Ok(())
}

pub async fn get_count_of_same_name(client: &PgPool, uid: i32) -> Result<i64> {
    let same_name_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM freshman.students
        WHERE name = (SELECT name FROM freshman.students WHERE uid = $1 LIMIT 1)",
    )
    .bind(uid)
    .fetch_one(client)
    .await?;
    Ok(same_name_count.0)
}
