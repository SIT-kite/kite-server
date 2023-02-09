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

use anyhow::Result;
use chrono::{DateTime, Local};
use sqlx::PgPool;

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

pub mod validate {
    use regex_macro::regex;

    pub fn check_username(account: &str) -> bool {
        let len = account.len() as i32;

        if ![4, 9, 10].contains(&len) {
            return false;
        }
        let regex = regex!(r"^((\d{2}6\d{6})|(\d{4})|(\d{6}[YGHE\d]\d{3}))$");
        return regex.is_match(&account.to_uppercase());
    }
}

pub async fn get(pool: &PgPool, uid: i32) -> Result<Option<User>> {
    sqlx::query_as("SELECT uid, account, create_time, role, is_block FROM user_account WHERE uid = $1 LIMIT 1;")
        .bind(uid)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn query(pool: &PgPool, account: &str) -> Result<Option<User>> {
    sqlx::query_as("SELECT uid, account, create_time, role, is_block FROM user_account WHERE account = $1 LIMIT 1;")
        .bind(account)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn create(pool: &PgPool, account: &str, name: &str) -> Result<User> {
    sqlx::query_as(
        "INSERT INTO user_account (account, name) VALUES($1, $2) \
        ON CONFLICT (account) DO UPDATE SET account = $1, name = $2 \
        RETURNING uid, account, create_time, role, is_block;",
    )
    .bind(account)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
