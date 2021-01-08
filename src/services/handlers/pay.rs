//! This module includes interfaces for querying electricity bill and expenses record.
use crate::error::Result;
use crate::models::pay::BalanceManager;
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};
use chrono::Duration;
use std::ops::Sub;

/**********************************************************************
    Interfaces in this module:
    query_room_balance()         <-- GET  /pay/room/{room}
    query_consumption_bill()     <-- GET  /pay/consumption/{studentId}
*********************************************************************/

#[get("/pay/room/{room}")]
pub async fn query_room_balance(app: web::Data<AppState>, form: web::Path<i32>) -> Result<HttpResponse> {
    let room = form.into_inner();
    let manager = BalanceManager::new(&app.pool);
    let result = manager.query_last_balance(room).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}

#[get("/pay/room/{room}/rank")]
pub async fn query_room_consumption_rank(
    app: web::Data<AppState>,
    form: web::Path<i32>,
) -> Result<HttpResponse> {
    let room = form.into_inner();
    let manager = BalanceManager::new(&app.pool);
    let result = manager.query_recent_consumption_rank(room).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}

#[derive(serde::Deserialize)]
pub struct DateRange {
    start: Option<String>,
    end: Option<String>,
}

#[get("/pay/room/{room}/bill/days")]
pub async fn query_room_bills_by_day(
    app: web::Data<AppState>,
    form: web::Path<i32>,
    parameters: web::Query<DateRange>,
) -> Result<HttpResponse> {
    let room = form.into_inner();
    let parameters = parameters.into_inner();
    let start_date = parameters.start.unwrap_or_else(|| {
        chrono::Local::today()
            .sub(Duration::days(7))
            .format("%Y-%m-%d")
            .to_string()
    });
    let end_date = parameters
        .end
        .unwrap_or_else(|| chrono::Local::today().format("%Y-%m-%d").to_string());

    let manager = BalanceManager::new(&app.pool);
    let result = manager
        .query_statistics_by_day(room, start_date, end_date)
        .await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}

#[get("/pay/room/{room}/bill/hours")]
pub async fn query_room_bills_by_hour(
    app: web::Data<AppState>,
    form: web::Path<i32>,
) -> Result<HttpResponse> {
    let room = form.into_inner();
    let manager = BalanceManager::new(&app.pool);

    let start_time = chrono::offset::Local::now().sub(Duration::days(1));
    let end_time = chrono::Local::now();
    let result = manager.query_balance_by_hour(room, start_time, end_time).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}
