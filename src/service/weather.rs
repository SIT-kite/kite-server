use poem::web::{Data, Json, Path, Query};
use poem::{handler, Result};
use serde::Deserialize;
use sqlx::PgPool;

use crate::model::weather;
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct WeatherParam {
    lang: Option<i32>,
}

#[handler]
pub async fn get_weather(pool: Data<&PgPool>, Path(campus): Path<i32>, Query(param): Query<WeatherParam>) -> Result<Json<serde_json::Value>> {
    let data = weather::get_recent_weather(&pool, campus, param.lang.unwrap_or(1)).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}
