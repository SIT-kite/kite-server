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

use poem::handler;
use poem::web::{Data, Json, Path};
use sqlx::PgPool;

use kite::model::balance as model;

use crate::error::Result;
use crate::response::ApiResponse;

#[handler]
pub async fn query_room_balance(pool: Data<&PgPool>, Path(room): Path<i32>) -> Result<Json<serde_json::Value>> {
    let data = model::get_latest_balance(&pool, room).await?;

    let content: serde_json::Value = if let Some(data) = data {
        ApiResponse::normal(data).into()
    } else {
        ApiResponse::<()>::fail(404, "No such room.".to_string()).into()
    };
    Ok(Json(content))
}
