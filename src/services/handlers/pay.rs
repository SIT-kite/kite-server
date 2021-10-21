//! This module includes interfaces for querying electricity bill and expenses record.
use std::ops::Sub;

use actix_web::{get, HttpResponse, web};
use chrono::Duration;

use crate::bridge::{ExpenseRequest, HostError, RequestFrame, RequestPayload, ResponsePayload};
use crate::error::ApiError;
use crate::error::Result;
use crate::models::{CommonError, PageView};
use crate::models::pay::BalanceManager;
use crate::models::user::Person;
use crate::services::AppState;
use crate::services::JwtToken;
use crate::services::response::ApiResponse;

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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseQuery {
    start_time: Option<String>,
    end_time: Option<String>,
}

#[get("/pay/expense")]
pub async fn query_expense(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    page: web::Query<PageView>,
    query: web::Query<ExpenseQuery>,
) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;
    let account = identity.student_id;
    let password = identity.oa_secret;

    let request = ExpenseRequest {
        account,
        password,
        page: Some(page.index()),
        start_time: query.start_time.clone(),
        end_time: query.end_time.clone(),
    };

    let agents = &app.agents;
    let payload = RequestPayload::CardExpense(request);
    let request = RequestFrame::new(payload);

    let response = agents.request(request).await??;
    if let ResponsePayload::CardExpense(result) = response {
        Ok(HttpResponse::Ok().json(ApiResponse::normal(result)))
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}
