use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::bridge::{
    AgentManager, ExpensePage, ExpenseRecord, ExpenseRequest, HostError, RequestFrame, RequestPayload,
    ResponsePayload,
};
use crate::error::{ApiError, Result};
use crate::models::PageView;

pub async fn save_expense_record(pool: &PgPool, student_id: &str, record: &ExpenseRecord) -> Result<()> {
    sqlx::query("CALL pay.insert_expense_record($1, $2, $3, $4);")
        .bind(student_id)
        .bind(record.ts)
        .bind(record.amount)
        .bind(&record.address)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn save_expense_records(
    pool: &PgPool,
    student_id: &str,
    records: &Vec<ExpenseRecord>,
) -> Result<()> {
    for r in records {
        save_expense_record(pool, student_id, r).await?;
    }
    Ok(())
}

pub async fn query_expense_records(
    pool: &PgPool,
    student_id: &str,
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    page: PageView,
) -> Result<Vec<ExpenseRecord>> {
    let records = sqlx::query_as(
        "SELECT student_id, ts, amount, address
                    FROM pay.expense_record
                    WHERE student_id = $1
                      AND ts BETWEEN $2 AND $3
                    ORDER BY ts DESC
                    LIMIT $4 OFFSET $5;",
    )
    .bind(student_id)
    .bind(start_time)
    .bind(end_time)
    .bind(page.count(20) as i32)
    .bind(page.offset(20) as i32)
    .fetch_all(pool)
    .await?;

    Ok(records)
}

pub async fn query_last_record_ts(pool: &PgPool, student_id: &str) -> Result<Option<DateTime<Local>>> {
    let ts: Option<(DateTime<Local>,)> = sqlx::query_as(
        "SELECT ts
            FROM pay.expense
            WHERE student_id = $1
            ORDER BY ts DESC
            LIMIT 1;",
    )
    .bind(student_id)
    .fetch_optional(pool)
    .await?;
    Ok(ts.map(|x| x.0))
}

pub async fn request_expense_page(
    agents: &AgentManager,
    request: &ExpenseRequest,
) -> Result<ExpensePage> {
    let max_remain = 3;
    let mut remain = max_remain;
    let mut wait_time = 2;

    loop {
        let request = request.clone();
        let payload = RequestPayload::CardExpense(request);
        let request = RequestFrame::new(payload);
        let response = agents.request(request).await?;

        match response {
            Ok(response) => {
                return if let ResponsePayload::CardExpense(result) = response {
                    Ok(result)
                } else {
                    Err(ApiError::new(HostError::Mismatched))
                }
            }
            Err(e) => {
                println!(
                    "Request expense page error(remind {}/{}): {:?}",
                    remain, max_remain, e
                );
                if remain == 0 {
                    return Err(ApiError::from(e));
                } else {
                    remain -= 1;
                    let _ = tokio::time::sleep(tokio::time::Duration::from_secs(wait_time));
                    wait_time <<= 2;
                }
            }
        }
    }
}
