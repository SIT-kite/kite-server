//! This module includes interfaces for querying electricity bill and expenses record.
use std::ops::Sub;
use std::str::FromStr;

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
use crate::models::user::{Identity, Person};
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
) -> Result<()> {
    use crate::models::pay::{request_expense_page, save_expense_records};

    let end_time = chrono::Local::today();
    let expense_request = ExpenseRequest {
        account: account.to_string(),
        password: credential.to_string(),
        page: Some(1),
        start_time: Some("20211001".to_string()),
        end_time: Some(end_time.format("%Y%m%d").to_string()),
    };
    let first_page = request_expense_page(&agents, &expense_request).await?;
    save_expense_records(&pool, account, &first_page.records).await;

    for i in 2..=first_page.page.total {
        let mut request = expense_request.clone();
        let agents = agents.clone();
        let pool = pool.clone();
        let account = account.to_string();

        tokio::spawn(async move {
            request.page = Some(i as u16);
            let page = request_expense_page(&agents, &request).await;

            match page {
                Ok(page) => {
                    save_expense_records(&pool, &account, &page.records).await;
                }
                Err(e) => println!("Fetch expense records error: {:?}", e),
            }
        });
    }
    Ok(())
}

pub async fn fetch_expense_in_parallel(identity: Identity, app: web::Data<AppState>) -> Result<()> {
    tokio::spawn(async move {
        let pool = app.pool.clone();
        let agents = app.agents.clone();

        fetch_all_expense_records(pool, agents, &identity.student_id, &identity.oa_secret).await;
    });
    Ok(())
}

pub async fn fetch_expense_in_iteration(identity: Identity, app: web::Data<AppState>) -> Result<()> {
    tokio::spawn(fetch_expense_iteratively(identity, app));

    Ok(())
}

pub async fn fetch_expense_iteratively(identity: Identity, app: web::Data<AppState>) -> Result<()> {
    use crate::models::pay::{
        query_last_record_ts, request_expense_page, save_expense_record, save_expense_records,
    };

    let agents = &app.agents;
    let pool = &app.pool;

    let end_time = chrono::Local::today();
    let mut expense_request = ExpenseRequest {
        account: identity.student_id,
        password: identity.oa_secret,
        page: Some(1),
        start_time: Some("20211001".to_string()),
        end_time: Some(end_time.format("%Y%m%d").to_string()),
    };
    let mut page = 1;
    let mut total_page = 1;
    let recent_record_in_db = query_last_record_ts(&pool, &expense_request.account)
        .await?
        .unwrap_or_else(|| parse_date_from_str("1970-01-01 08:00:00").unwrap());

    'OUTER: while page <= total_page {
        expense_request.page = Some(page);
        let current_page = request_expense_page(&agents, &expense_request).await;
        match current_page {
            Ok(current_page) => {
                total_page = current_page.page.total as u16;

                for r in current_page.records {
                    if r.ts < recent_record_in_db {
                        break 'OUTER;
                    }
                    save_expense_record(&pool, &expense_request.account, &r).await;
                }
            }
            Err(e) => println!(
                "Fetch expense records error ({}, page = {}): {:?}",
                expense_request.account, page, e
            ),
        }
        page += 1;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ExpenseFetchQuery {
    mode: u8,
}

/// 并发地刷新页面去请求爬虫刷新数据库
#[post("/pay/expense/fetch")]
pub async fn fetch_expense(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    query: web::Query<ExpenseFetchQuery>,
) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

    match query.into_inner().mode {
        1 => fetch_expense_in_parallel(identity, app).await?,
        2 => fetch_expense_in_iteration(identity, app).await?,
        _ => return Err(ApiError::new(CommonError::Parameter)),
    };
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
