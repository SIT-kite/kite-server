use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::weather::get_weather_from_db;
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[derive(Serialize, Deserialize)]
pub struct WeatherQuery {
    pub campus: i32,
}

#[get("/weather/now")]
pub async fn get_weather(
    app: web::Data<AppState>,
    params: web::Query<WeatherQuery>,
) -> Result<HttpResponse> {
    let params = params.into_inner();
    let pool = &app.pool;
    let result = get_weather_from_db(pool, params.campus).await?;
    let response = serde_json::json!({
        "weather": result,
    });
    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
}
