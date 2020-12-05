use crate::error::{ApiError, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Available power
    pub power: f32,
    /// Last update time
    pub ts: NaiveDateTime,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
/// Electricity usage statistics
pub struct ElectricityBill {
    /// Room Id
    pub room: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
    #[serde(rename = "startAt")]
    pub start_at: NaiveDateTime,
    #[serde(rename = "endAt")]
    pub end_at: NaiveDateTime,
}

pub enum AggregationLevel {
    Time,
    Hour,
    Day,
    Week,
    Month,
}

pub struct BalanceManager<'a> {
    db: &'a sqlx::PgPool,
}

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum BalanceError {
    #[error("无对应房间数据")]
    NoSuchRoom = 200,
    #[error("房间格式不正确")]
    InvalidRoomId = 201,
}

impl<'a> BalanceManager<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn query_last_balance(self, room: i32) -> Result<ElectricityBalance> {
        let balance: Option<ElectricityBalance> = sqlx::query_as(
            "SELECT room, total_balance AS balance, CAST(total_balance / 0.6 AS real) AS power, ts
                FROM dormitory.balance WHERE room = $1 ORDER BY ts DESC LIMIT 1",
        )
        .bind(room)
        .fetch_optional(self.db)
        .await?;

        balance.ok_or(ApiError::new(BalanceError::NoSuchRoom))
    }

    // pub fn query_bill_today(self, room: String) -> ElectricityBill {}
    // pub fn query_balance_statistics(self, room: String, level: AggregationLevel) {}
}
