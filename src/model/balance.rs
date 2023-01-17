use bincode::{Decode, Encode};
use chrono::{DateTime, Local};
use sqlx::FromRow;

#[derive(Encode, Decode, FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Last update time
    #[bincode(with_serde)]
    pub ts: DateTime<Local>,
}

/// Electricity usage statistics by day
#[derive(FromRow)]
pub struct DailyElectricityBill {
    /// Date string in 'yyyy-mm-dd'
    pub date: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Electricity usage statistics by hour
#[derive(FromRow)]
pub struct HourlyElectricityBill {
    /// Hour string in 'yyyy-mm-dd HH24:00'
    pub time: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Rank of recent-24hour consumption
#[derive(Clone, Encode, Decode, FromRow)]
pub struct RecentConsumptionRank {
    /// Consumption in last 24 hours.
    pub consumption: f32,
    /// Rank
    pub rank: i32,
    /// Total room count
    pub room_count: i32,
}
