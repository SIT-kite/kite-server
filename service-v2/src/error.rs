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

use poem::error::ResponseError;
use poem::http::StatusCode;
use serde::ser::StdError;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, serde::Serialize)]
pub struct ApiError {
    pub code: u16,
    pub msg: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl StdError for ApiError {}

impl ResponseError for ApiError {
    fn status(&self) -> StatusCode {
        StatusCode::OK
    }

    /// Convert this error to a HTTP response.
    fn as_response(&self) -> poem::Response
    where
        Self: StdError + Send + Sync + 'static,
    {
        poem::Response::builder()
            .status(self.status())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&self).unwrap())
    }
}

impl ApiError {
    pub fn new<T: std::error::Error>(sub_err: T) -> Self {
        Self {
            code: 1,
            msg: Some(sub_err.to_string()),
        }
    }

    pub fn custom(code: u16, msg: &str) -> Self {
        Self {
            code,
            msg: Some(msg.to_string()),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            code: 1,
            msg: Some(value.to_string()),
        }
    }
}
