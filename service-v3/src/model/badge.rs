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

use chrono::{DateTime, Local};

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
    pub card: i32,
    /// 操作时间
    pub ts: DateTime<Local>,
}
