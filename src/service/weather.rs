use poem::web::{Data, Json, Path};
use poem::{handler, Result};
use sqlx::PgPool;

use crate::model::weather;
use crate::response::ApiResponse;

#[handler]
pub async fn get_weather(pool: Data<&PgPool>, Path(campus): Path<i32>) -> Result<Json<serde_json::Value>> {
    let data = weather::get_recent_weather(&pool, campus).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}
