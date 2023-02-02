/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
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
use regex_macro::regex;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    /// 用户 ID
    pub uid: i32,
    /// 用户学号
    pub account: String,
    /// 用户创建日期
    pub create_time: DateTime<Local>,
    /// 用户角色
    pub role: i32,
    /// 账户是否被禁用
    pub is_block: bool,
}

pub struct Validator;

impl Validator {
    pub fn validate_username(account: &str) -> bool {
        if account.len() != 4 && account.len() != 10 && account.len() != 9 {
            return false;
        }
        let regex = regex!(r"^((\d{2}6\d{6})|(\d{4})|(\d{6}[YGHE\d]\d{3}))$");
        return regex.is_match(&account.to_uppercase());
    }
}
