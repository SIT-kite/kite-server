/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use bincode::{Decode, Encode};
use chrono::{DateTime, Local};

use sqlx::FromRow;

#[derive(Clone, Encode, Decode, FromRow)]
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
#[derive(Clone, Encode, Decode, FromRow)]
pub struct DailyElectricityBill {
    /// Date string in 'yyyy-mm-dd'
    pub date: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Electricity usage statistics by hour
#[derive(Clone, Encode, Decode, FromRow)]
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
