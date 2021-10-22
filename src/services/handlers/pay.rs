//! This module includes interfaces for querying electricity bill and expenses record.
use std::ops::Sub;

use actix_web::{get, post, web, HttpResponse};
use chrono::{
    Date, DateTime, Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone,
};
use sqlx::PgPool;

use crate::services::response::ApiResponse;
use crate::services::AppState;

use crate::bridge::{
    AgentManager, ExpenseRequest, HostError, RequestFrame, RequestPayload, ResponsePayload,
};
use crate::error::ApiError;
use crate::error::Result;
use crate::models::pay::BalanceManager;
use crate::models::user::Person;
use crate::models::{CommonError, PageView};
use crate::services::JwtToken;

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

pub async fn fetch_all_expense_records(
    pool: PgPool,
    agents: AgentManager,
    account: &str,
    credential: &str,
) {
    use crate::models::pay::{request_expense_page, save_expense_records};

    let end_time = chrono::Local::today();
    let expense_request = ExpenseRequest {
        account: account.to_string(),
        password: credential.to_string(),
        page: Some(1),
        start_time: Some("20100101".to_string()),
        end_time: Some(end_time.format("%Y%m%d").to_string()),
    };
    let first_page = request_expense_page(&agents, &expense_request).await.unwrap();
    save_expense_records(&pool, account, &first_page.records).await;

    for i in 2..=first_page.page.total {
        let mut request = expense_request.clone();
        let agents = agents.clone();
        let pool = pool.clone();
        let account = account.to_string();

        // tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        tokio::spawn(async move {
            request.page = Some(i as u16);
            let page = request_expense_page(&agents, &request).await;

            match page {
                Ok(page) => save_expense_records(&pool, &account, &page.records).await,
                Err(e) => println!("Fetch expense records error: {:?}", e),
            }
        });
    }
}

/// 一步步地刷新页面去请求爬虫刷新数据库
#[post("/pay/expense/fetch")]
pub async fn fetch_expense(token: Option<JwtToken>, app: web::Data<AppState>) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

    tokio::spawn(async move {
        let pool = app.pool.clone();
        let agents = app.agents.clone();

        fetch_all_expense_records(pool, agents, &identity.student_id, &identity.oa_secret).await;
    });
    Ok(HttpResponse::Ok().json(ApiResponse::empty()))
}

fn parse_date_from_str(s: &str) -> Result<DateTime<Local>> {
    let offset = FixedOffset::east(8 * 3600);
    // 2001-07-08 00:34:60
    let ndt = offset.datetime_from_str(s, "%F %T");

    ndt.map(|d| DateTime::<Local>::from(d))
        .map_err(|_| ApiError::new(CommonError::Parameter))
}

/// 请求消费查询
#[get("/pay/expense")]
pub async fn query_expense(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    page: web::Query<PageView>,
    query: web::Query<ExpenseQuery>,
) -> Result<HttpResponse> {
    use crate::models::pay::query_expense_records;
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

    let query = query.into_inner();
    let start_time = query
        .start_time
        .map(|date| parse_date_from_str(&date))
        .unwrap_or_else(|| parse_date_from_str("2010-01-01 00:00:00"))?;
    let end_time = query
        .end_time
        .map(|date| parse_date_from_str(&date))
        .unwrap_or_else(|| Ok(Local::now()))?;

    let records = query_expense_records(
        &app.pool,
        &identity.student_id,
        start_time,
        end_time,
        page.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::normal(serde_json::json!({ "records": records }))))
}
