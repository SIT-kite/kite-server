use crate::error::{ApiError, Result};
use chrono::{DateTime, Local};
use num_derive::ToPrimitive;
use rsa::{pkcs8::FromPrivateKey, PaddingScheme, PublicKey, RsaPrivateKey};
use sqlx::PgPool;

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum LibraryError {
    #[error("找不到记录")]
    NoSuchItem = 200,
    #[error("当日已满")]
    AlreadyFull = 201,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Application {
    /// 预约编号
    pub id: i32,
    /// 预约场次. 格式为 `yyMMdd` + 场次 (1 上午, 2 下午, 3 晚上）
    pub period: i32,
    /// 学号/工号
    pub user: String,
    /// 场次下座位号
    pub index: i32,
    /// 预约状态
    pub status: i32,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Notice {
    /// 时间戳
    pub ts: DateTime<Local>,
    /// 内容
    pub html: String,
}

#[derive(sqlx::FromRow)]
pub struct Status {
    /// 预约场次
    pub period: i32,
    /// 已预约人数
    pub applied: i32,
}

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
pub async fn get_applications(pool: &PgPool, period: Option<i32>, user: Option<String>) -> Result<Vec<Application>> {
    if (period.is_none() && user.is_none()) || (period.is_some() && user.is_some()) {
        return Ok(vec![]);
    }
    let applications = sqlx::query_as(
        "SELECT id, period, \"user\", index, status
        FROM library.application_view
        WHERE (period = $1 OR $1 IS NULL)
            AND (\"user\" = $2 OR $2 IS NULL);",
    )
    .bind(period)
    .bind(user)
    .fetch_all(pool)
    .await?;
    Ok(applications)
}

/// 查询单条申请记录
pub async fn query_application_by_id(pool: &PgPool, id: i32) -> Result<Option<Application>> {
    let result = sqlx::query_as(
        "SELECT id, period, \"user\", index, status
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
pub async fn get_code(pool: &PgPool, id: i32, private_key: &str) -> Result<String> {
    let application = query_application_by_id(pool, id)
        .await?
        .ok_or(ApiError::new(LibraryError::NoSuchItem))?;

    let content = serde_json::to_string(&application).unwrap();

    let mut rng = rand::thread_rng();
    let key = RsaPrivateKey::from_pkcs8_pem(private_key).expect("Invalid Rsa key.");
    let result = key
        .encrypt(&mut rng, PaddingScheme::PKCS1v15Encrypt, content.as_bytes())
        .unwrap();

    Ok(base64::encode(result))
}

#[derive(sqlx::FromRow)]
pub struct ApplyResult {
    pub id: Option<i32>,
    pub index: i32,
}

/// 预约座位
pub async fn apply(pool: &PgPool, uid: i32, period: i32) -> Result<ApplyResult> {
    // library.apply(_uid int, _period int, max_seat int)
    let result: ApplyResult = sqlx::query_as("SELECT id, index FROM library.apply($1, $2, 275);")
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
pub async fn cancel(pool: &PgPool, apply_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM library.application WHERE id = $1;")
        .bind(apply_id)
        .execute(pool)
        .await?;
    Ok(())
}
