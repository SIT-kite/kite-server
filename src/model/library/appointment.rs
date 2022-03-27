use super::LibraryError;
use super::{Application, Notice, Status};
use crate::error::{ApiError, Result};
use chrono::{DateTime, Local};
use rsa::{pkcs8::FromPrivateKey, Hash, PaddingScheme, RsaPrivateKey};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// 获取图书馆公告
pub async fn get_notice(pool: &PgPool) -> Result<Option<Notice>> {
    let notice: Option<Notice> = sqlx::query_as("SELECT html, ts FROM library.notice ORDER BY ts DESC LIMIT 1;")
        .fetch_optional(pool)
        .await?;
    Ok(notice)
}

/// 获取预约状态
pub async fn get_status(pool: &PgPool, date: i32) -> Result<Vec<Status>> {
    let result = sqlx::query_as(
        "SELECT generate_series AS period, COALESCE(applied, 0) AS applied 
        FROM generate_series($1, $2)
        LEFT JOIN (
            SELECT period, CAST(COUNT(*) AS int) AS applied
            FROM library.application
            GROUP BY period
            ORDER BY period
        ) AS applied_count
        ON period = generate_series;",
    )
    .bind(date * 10 + 1)
    .bind(date * 10 + 3)
    .fetch_all(pool)
    .await?;
    Ok(result)
}

/// 获取预约列表
///
/// 请不要同时提供或不提供 period/user 参数.
pub async fn get_applications(
    pool: &PgPool,
    period: Option<i32>,
    user: Option<String>,
    uid: Option<i32>,
    date: Option<i32>,
) -> Result<Vec<Application>> {
    if date.is_none() && ((period.is_none() && user.is_none()) || (period.is_some() && user.is_some())) {
        return Ok(vec![]);
    }
    let applications = sqlx::query_as(
        "SELECT id, period, \"user\", index, status, ts
        FROM library.application_view
        WHERE (period = $1 OR $1 IS NULL)
            AND (\"user\" = $2 OR $2 IS NULL)
            AND (uid = $3 OR $3 IS NULL)
            AND (period / 10 = $4 OR $4 IS NULL);",
    )
    .bind(period)
    .bind(user)
    .bind(uid)
    .bind(date)
    .fetch_all(pool)
    .await?;
    Ok(applications)
}

/// 查询单条申请记录
pub async fn query_application_by_id(pool: &PgPool, id: i32) -> Result<Option<Application>> {
    let result = sqlx::query_as(
        "SELECT id, period, \"user\", index, status, ts
        FROM library.application_view
        WHERE id = $1
        LIMIT 1;",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(result)
}

/// 生成加密的二维码信息
pub fn generate_sign(application: &Application, private_key: &str, ts: &DateTime<Local>) -> Result<String> {
    let clear_text = format!(
        "{}|{}|{}|{}|{}",
        &application.period,
        &application.user,
        &application.index,
        &application.id,
        ts.timestamp(),
    );

    let digest = Sha256::digest(clear_text.as_bytes()).to_vec();
    let key = RsaPrivateKey::from_pkcs8_pem(private_key).expect("Invalid Rsa key.");

    let result = key
        .sign(PaddingScheme::new_pkcs1v15_sign(Some(Hash::SHA2_256)), &digest)
        .unwrap();

    Ok(base64::encode(result))
}

#[derive(sqlx::FromRow)]
pub struct ApplyResult {
    pub id: Option<i32>,
    pub index: i32,
    pub is_exist: bool,
}

/// 预约座位
pub async fn apply(pool: &PgPool, uid: i32, period: i32) -> Result<ApplyResult> {
    // library.apply(_uid int, _period int, max_seat int)
    let result: ApplyResult = sqlx::query_as("SELECT id, index, is_exist FROM library.apply($1, $2, 275);")
        .bind(uid)
        .bind(period)
        .fetch_one(pool)
        .await?;
    if result.id.is_none() {
        return Err(ApiError::new(LibraryError::AlreadyFull));
    }
    Ok(result)
}

/// 更新预约状态
pub async fn update_application(pool: &PgPool, apply_id: i32, status: i32) -> Result<()> {
    sqlx::query("UPDATE library.application SET status = $1 WHERE id = $2;")
        .bind(status)
        .bind(apply_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 取消预约
pub async fn cancel(pool: &PgPool, apply_id: i32, uid: Option<i32>) -> Result<()> {
    sqlx::query("DELETE FROM library.application WHERE id = $1 AND (uid = $2 OR $2 IS NULL);")
        .bind(apply_id)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}
