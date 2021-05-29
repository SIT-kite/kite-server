use crate::error::Result;
use crate::models::PageView;
use chrono::{DateTime, Local};
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

///pages
///
/// The pages in OA portal.
#[derive(sqlx::FromRow, Serialize)]
pub struct Page {
    pub title: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub publish_date: Option<NaiveDate>,
    pub update_date: Option<NaiveDate>,
    pub link_count: Option<i16>,
    pub content: Option<String>,
}

pub async fn query_page(pool: &PgPool, query: &str, page: &PageView) -> Result<Vec<Page>> {
    let result = sqlx::query_as(
        "SELECT title, host, path, publish_date, update_date, link_count, content
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

// ///attachments
// ///
// /// The pages in OA portal.
// #[derive(sqlx::FromRow, Serialize())]
// pub struct Attachment {
//     pub id: i32,
//     pub title: Option<String>,
//     pub host: Option<String>,
//     pub path: Option<String>,
//     pub size: Option<i32>,
//     pub local_name: Option<String>,
//     pub checksum: Option<String>,
//     pub referer: Option<String>,
// }
//
// pub async fn query_attachment(pool: &PgPool, query: &str, page: &PageView) -> Result<Vec<Attachment>> {
//     let result = sqlx::query_as(
//         "",
//     )
//         .fetch_all(pool)
//         .await?;
//
//     Ok(result)
// }