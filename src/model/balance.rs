use chrono::{DateTime, Local};

#[derive(serde::Serialize, sqlx::FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
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
#[serde(rename_all = "camelCase")]
pub struct RecentConsumptionRank {
    /// Consumption in last 24 hours.
    pub consumption: f32,
    /// Rank
    pub rank: i32,
    /// Total room count
    pub room_count: i32,
}
