use chrono::{DateTime, Local};

use crate::error::{ApiError, Result};

#[derive(serde::Serialize, sqlx::FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Available power
    pub power: f32,
    /// Last update time
    pub ts: DateTime<Local>,
}

/// Electricity usage statistics by day
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct DailyElectricityBill {
    /// Date string in 'yyyy-mm-dd'
    pub date: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Electricity usage statistics by hour
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct HourlyElectricityBill {
    /// Hour string in 'yyyy-mm-dd HH24:00'
    pub time: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Rank of recent-24hour consumption
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct RecentConsumptionRank {
    /// Consumption in last 24 hours.
    pub consumption: f32,
    /// Rank
    pub rank: i32,
    /// Total room count
    pub room_count: i32,
}

pub struct BalanceManager<'a> {
    db: &'a sqlx::PgPool,
}

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum BalanceError {
    #[error("无对应房间数据")]
    NoSuchRoom = 200,
}

impl<'a> BalanceManager<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn query_last_balance(self, room: i32) -> Result<ElectricityBalance> {
        sqlx::query_as(
            "SELECT room, total_balance AS balance, CAST(total_balance / 0.6 AS real) AS power, ts
                FROM dormitory.balance WHERE room = $1 ORDER BY ts DESC LIMIT 1",
        )
        .bind(room)
        .fetch_optional(self.db)
        .await?
        .ok_or_else(|| ApiError::new(BalanceError::NoSuchRoom))
    }

    pub async fn query_statistics_by_day(
        self,
        room: i32,
        start_date: String,
        end_date: String,
    ) -> Result<Vec<DailyElectricityBill>> {
        let bills = sqlx::query_as(
            "SELECT d.day AS date, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM
                    (SELECT to_char(day_range, 'yyyy-MM-dd') AS day FROM generate_series($1::date,  $2::date, '1 day') AS day_range) d
                LEFT JOIN (
                    SELECT * FROM dormitory.get_consumption_report_by_day($1::date, CAST($2::date + '1 day'::interval AS date), $3)
                ) AS records
                ON d.day = records.day;")
            .bind(start_date)
            .bind(end_date)
            .bind(room)
            .fetch_all(self.db)
            .await?;

        Ok(bills)
    }

    pub async fn query_balance_by_hour(
        self,
        room: i32,
        start_ts: DateTime<Local>,
        end_ts: DateTime<Local>,
    ) -> Result<Vec<HourlyElectricityBill>> {
        let bills = sqlx::query_as(
            "SELECT h.hour AS time, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM
                    (
                        SELECT to_char(hour_range, 'yyyy-MM-dd HH24:00') AS hour
                        FROM generate_series($1::timestamptz, $2::timestamptz, '1 hour') AS hour_range
                    ) h
                LEFT JOIN (
                    SELECT * FROM dormitory.get_consumption_report_by_hour($1::timestamptz, $2::timestamptz, $3)
                ) AS records
                ON h.hour = records.hour;"
        )
            .bind(start_ts)
            .bind(end_ts)
            .bind(room)
            .fetch_all(self.db)
            .await?;
        Ok(bills)
    }

    pub async fn query_recent_consumption_rank(self, room: i32) -> Result<RecentConsumptionRank> {
        sqlx::query_as(
            "SELECT room, consumption, rank, (SELECT CAST(COUNT(*) AS integer) FROM dormitory.rooms) AS room_count
                FROM dormitory.rank_last_24hour_consumption()
                WHERE room = $1 LIMIT 1",
        )
            .bind(room)
            .fetch_optional(self.db)
            .await?
            .ok_or_else(|| ApiError::new(BalanceError::NoSuchRoom))
    }
}
