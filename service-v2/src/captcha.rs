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
use poem::web::Json;

use crate::error::Result;
use crate::response::ApiResponse;

#[handler]
pub async fn recognize_captcha(body: String) -> Result<Json<serde_json::Value>> {
    async fn inner_recognize(image_in_base64: String) -> anyhow::Result<String> {
        let image = base64::decode(image_in_base64)?;
        let text = captcha::async_recognize(image).await?;

        tracing::info!("Captcha result: {text}");
        Ok(text)
    }

    let result = if !body.is_empty() {
        match inner_recognize(body).await {
            Ok(text) => ApiResponse::normal(text),
            Err(e) => ApiResponse::fail(1, e.to_string()),
        }
    } else {
        let err = "No request body provided.".to_string();
        ApiResponse::fail(1, err)
    };

    Ok(Json(result.into()))
}
