use chrono::{Date, Duration, Local};
use poem::web::Data;
use poem::Result;
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};
use sqlx::PgPool;
use std::ops::Sub;

use crate::model::electricity;
use crate::response::ApiResponse;
use super::ApiTags;

/**********************************************************************
 Interfaces in this module:
 query_room_balance()         <-- GET /electricity/room/{room}
 query_room_consumption_rank()<-- GET /electricity/room/{room}/rank
 query_room_bills_by_day()    <-- GET /electricity/room/{room}/bill/days
 query_room_bills_by_hour()   <-- GET /electricity/room/{room}/bill/hours
*********************************************************************/

pub struct ElectricityApi;

#[OpenApi]
impl ElectricityApi {
    #[oai(path = "/electricity/room/:room", method = "get", tag = "ApiTags::Electricity")]
    pub async fn query_room_balance(&self, pool: Data<&PgPool>, room: Path<i32>) -> Result<Json<serde_json::Value>> {
        let data = electricity::query_last_balance(&pool, room.0).await?;
        let response: serde_json::Value = ApiResponse::normal(data).into();

        Ok(Json(response))
    }

    #[oai(path = "/electricity/room/:room/rank", method = "get", tag = "ApiTags::Electricity")]
    pub async fn query_room_consumption_rank(
        &self,
        pool: Data<&PgPool>,
        room: Path<i32>,
    ) -> Result<Json<serde_json::Value>> {
        let data = electricity::query_recent_consumption_rank(&pool, room.0).await?;
        let response: serde_json::Value = ApiResponse::normal(data).into();

        Ok(Json(response))
    }

    #[oai(path = "/electricity/room/:room/bill/days", method = "get", tag = "ApiTags::Electricity")]
    pub async fn query_room_bills_by_day(
        &self,
        pool: Data<&PgPool>,
        room: Path<i32>,
        start: Query<Option<String>>,
        end: Query<Option<String>>,
    ) -> Result<Json<serde_json::Value>> {
        let today = chrono::Local::today();
        let to_str = |x: Date<Local>| x.format("%Y-%m-%d").to_string();

        let start_date = start.0.unwrap_or_else(|| to_str(today.sub(Duration::days(7))));
        let end_date = end.0.unwrap_or_else(|| to_str(today));

        let data = electricity::query_statistics_by_day(&pool, room.0, start_date, end_date).await?;
        let response: serde_json::Value = ApiResponse::normal(data).into();

        Ok(Json(response))
    }

    #[oai(path = "/electricity/room/:room/bill/hours", method = "get", tag = "ApiTags::Electricity")]
    pub async fn query_room_bills_by_hour(
        &self,
        pool: Data<&PgPool>,
        room: Path<i32>,
    ) -> Result<Json<serde_json::Value>> {
        let now = chrono::Local::now();

        let start_time = now.sub(Duration::days(1));
        let end_time = now;

        let data = electricity::query_balance_by_hour(&pool, room.0, start_time, end_time).await?;
        let response: serde_json::Value = ApiResponse::normal(data).into();

        Ok(Json(response))
    }
}
