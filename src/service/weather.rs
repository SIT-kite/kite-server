use poem::web::Data;
use poem::Result;
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};
use sqlx::PgPool;

use crate::model::weather;
use crate::response::ApiResponse;
use super::ApiTags;

pub struct WeatherApi;

#[OpenApi]
impl WeatherApi {
    #[oai(path = "/weather/:campus", method = "get", tag = "ApiTags::Weather")]
    pub async fn get_weather(
        &self,
        pool: Data<&PgPool>,
        campus: Path<i32>,
        lang: Query<Option<i32>>,
    ) -> Result<Json<serde_json::Value>> {
        let data = weather::get_recent_weather(&pool, campus.0, lang.0.unwrap_or(1)).await?;
        let response: serde_json::Value = ApiResponse::normal(data).into();

        Ok(Json(response))
    }
}
