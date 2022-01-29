//! 扫福活动

use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::error::Result;

/// 识别结果
#[derive(num_derive::ToPrimitive, num_derive::FromPrimitive)]
enum ScanResult {
    /// 没有识别到校徽
    NoBadge = 1,
    /// 当日领福卡次数已达到限制
    ReachLimit = 2,
    /// 没有抽中
    NoCard = 3,
    /// 抽中了
    WinCard = 4,
}

/// 识别记录
#[derive(serde::Serialize)]
pub struct ScanRecord {
    /// 操作用户 ID
    pub uid: i32,
    /// 操作结果类型, 见 `ScanResult`
    pub result: i32,
    /// 卡片类型 （五种福卡之一）
    pub card: Option<i32>,
    /// 操作时间
    pub ts: DateTime<Local>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Card {
    /// 卡片类型 （五种福卡之一）
    pub card: Option<i32>,
    /// 操作时间
    pub ts: DateTime<Local>,
}

pub async fn get_cards_list(pool: &PgPool, uid: i32) -> Result<Vec<Card>> {
    let cards = sqlx::query_as("SELECT card, ts FROM fu.scan WHERE uid = $1 AND result = 3 AND card != 0;")
        .bind(uid)
        .fetch_all(pool)
        .await?;
    Ok(cards)
}
