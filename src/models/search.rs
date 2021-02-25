use crate::error::Result;
use crate::models::PageView;
use chrono::{DateTime, Local, NaiveDateTime};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, thiserror::Error, ToPrimitive)]
pub enum SearchError {
    #[error("该分类内容需要实名认证后查看")]
    NeedIdentity = 220,
}

/// Notice
///
/// The notices in OA portal.
#[derive(sqlx::FromRow, Serialize)]
pub struct Notice {
    pub url: String,
    pub title: String,
    pub publish_time: DateTime<Local>,
    pub department: String,
    pub author: Option<String>,
    pub sort: String,
    pub content: String,
}

pub async fn query_notice(pool: &PgPool, query: &str, page: &PageView) -> Result<Vec<Notice>> {
    let result = sqlx::query_as(
        "SELECT ('http://' || url) AS url, title, publish_time, department, author, sort, content
            FROM search.search_notice($1)
            ORDER BY publish_time DESC
            OFFSET $2 LIMIT $3;",
    )
    .bind(query)
    .bind(page.offset(20) as i32)
    .bind(page.count(20) as i32)
    .fetch_all(pool)
    .await?;

    Ok(result)
}
