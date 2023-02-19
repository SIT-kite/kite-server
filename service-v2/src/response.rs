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

use serde::Serialize;

#[derive(serde::Serialize)]
pub struct ApiResponse<T: Serialize> {
    code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(code: u16, msg: Option<String>, data: Option<T>) -> Self {
        Self { code, msg, data }
    }

    pub fn normal(data: T) -> Self {
        Self::new(0, None, Some(data))
    }

    pub fn empty() -> Self {
        Self::new(0, None, None)
    }

    pub fn fail(code: u16, msg: String) -> Self {
        Self::new(code, Some(msg), None)
    }
}

impl<T: Serialize> Into<serde_json::Value> for ApiResponse<T> {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}

impl<T> ToString for ApiResponse<T>
where
    T: Serialize,
{
    // Serialize
    fn to_string(&self) -> String {
        if let Ok(body_json) = serde_json::to_string(&self) {
            return body_json;
        }
        String::from("Critical: Could not serialize error message.")
    }
}
