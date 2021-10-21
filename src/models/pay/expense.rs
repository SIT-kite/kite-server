use chrono::{DateTime, Local};

#[derive(serde::Serialize, sqlx::FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct Expense {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Available power
    pub power: f32,
    /// Last update time
    pub ts: DateTime<Local>,
}
