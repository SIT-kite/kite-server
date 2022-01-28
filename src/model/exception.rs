use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::error::Result;

#[derive(Debug, serde::Deserialize)]
pub struct Exception {
    /// 错误信息 (异常里的一行文本描述)
    pub error: String,
    /// 发生时间
    #[serde(rename = "dateTime")]
    pub date_time: DateTime<Local>,
    /// 调用栈
    #[serde(rename = "stackTrace")]
    pub stack: String,
    /// 平台名称
    #[serde(rename = "platformType")]
    pub platform: String,
    /// 自定义参数
    #[serde(rename = "customParameters")]
    pub custom: serde_json::Value,
    /// 设备型号参数
    #[serde(rename = "deviceParameters")]
    pub device: serde_json::Value,
    /// 应用程序信息
    #[serde(rename = "applicationParameters")]
    pub application: serde_json::Value,
}

pub async fn save_exception(pool: &PgPool, exception: &Exception) -> Result<()> {
    sqlx::query(
        "INSERT INTO public.exception \
        (error, date_time, stack, platform, custom, device, application) \
        VALUES ($1, $2, $3, $4, $5, $6, $7);",
    )
    .bind(&exception.error)
    .bind(&exception.date_time)
    .bind(&exception.stack)
    .bind(&exception.platform)
    .bind(&exception.custom)
    .bind(&exception.device)
    .bind(&exception.application)
    .execute(pool)
    .await?;
    Ok(())
}
