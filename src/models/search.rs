use crate::error::Result;
use crate::models::PageView;
use chrono::{DateTime, Local, NaiveDate};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, thiserror::Error, ToPrimitive)]
pub enum SearchError {
    #[error("该分类内容需要实名认证后查看")]
    NeedIdentity = 220,
}

/// The notices in OA portal.
#[derive(sqlx::FromRow, Serialize)]
pub struct Notice {
    /// Id
    pub id: i32,
    /// Notice url in OA
    pub url: String,
    pub title: String,
    pub publish_time: DateTime<Local>,
    pub department: String,
    /// Author name (with teacher ID)
    pub author: Option<String>,
    pub sort: String,
    pub content: String,
}

pub async fn query_notice(pool: &PgPool, query: &str, page: &PageView) -> Result<Vec<Notice>> {
    let result = sqlx::query_as(
        "SELECT id, ('http://' || url) AS url, title, publish_time, department, author, sort, content
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

/// The pages in OA portal.
#[derive(sqlx::FromRow, Serialize)]
pub struct Page {
    /// Id
    pub id: i32,
    /// Page title
    pub title: Option<String>,
    /// Complete page url
    pub url: Option<String>,
    /// Published date for articles.
    pub publish_date: Option<NaiveDate>,
    /// Last update date by spider
    pub update_date: Option<NaiveDate>,
    /// Page content without any format
    pub content: Option<String>,
}

/// Page summary shown in search results.
#[derive(sqlx::FromRow, Serialize)]
pub struct PageSummary {
    /// Id
    pub id: i32,
    /// Page title
    pub title: Option<String>,
    /// Published date for articles.
    pub publish_date: Option<NaiveDate>,
    /// Page content summary
    pub summary: Option<String>,
}

pub async fn query_page(pool: &PgPool, query: &str, page: &PageView) -> Result<Vec<PageSummary>> {
    let result = sqlx::query_as(
        "SELECT id, title, ('http://' || uri) AS uri, publish_date, content AS summary
            FROM search.search_page($1)
            ORDER BY publish_date DESC
            OFFSET $2 LIMIT $3;",
    )
    .bind(query)
    .bind(page.offset(20) as i32)
    .bind(page.count(20) as i32)
    .fetch_all(pool)
    .await?;

    Ok(result)
}
