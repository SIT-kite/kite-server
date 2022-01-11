use chrono::{Date, Duration, Local};
use poem::web::{Data, Json, Path, Query};
use poem::{handler, Result};
use sqlx::PgPool;
use std::ops::Sub;

use crate::model::electricity;
use crate::response::ApiResponse;

/**********************************************************************
 Interfaces in this module:
 query_room_balance()         <-- GET /electricity/room/{room}
 query_room_consumption_rank()<-- GET /electricity/room/{room}/rank
 query_room_bills_by_day()    <-- GET /electricity/room/{room}/bill/days
 query_room_bills_by_hour()   <-- GET /electricity/room/{room}/bill/hours
*********************************************************************/

#[handler]
pub async fn query_room_balance(
    pool: Data<&PgPool>,
    Path(room): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    let data = electricity::query_last_balance(&pool, room).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[handler]
pub async fn query_room_consumption_rank(
    pool: Data<&PgPool>,
    Path(room): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    let data = electricity::query_recent_consumption_rank(&pool, room).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[derive(serde::Deserialize)]
pub struct DateRange {
    start: Option<String>,
    end: Option<String>,
}

#[handler]
pub async fn query_room_bills_by_day(
    pool: Data<&PgPool>,
    Path(room): Path<i32>,
    Query(parameters): Query<DateRange>,
) -> Result<Json<serde_json::Value>> {
    let today = chrono::Local::today();
    let to_str = |x: Date<Local>| x.format("%Y-%m-%d").to_string();

    let start_date = parameters
        .start
        .unwrap_or_else(|| to_str(today.sub(Duration::days(7))));
    let end_date = parameters.end.unwrap_or_else(|| to_str(today));

    let data = electricity::query_statistics_by_day(&pool, room, start_date, end_date).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[handler]
pub async fn query_room_bills_by_hour(
    pool: Data<&PgPool>,
    Path(room): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    let now = chrono::Local::now();

    let start_time = now.sub(Duration::days(1));
    let end_time = now;

    let data = electricity::query_balance_by_hour(&pool, room, start_time, end_time).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}
