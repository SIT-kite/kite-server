use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::error::Result;
use crate::model::PageView;

/// Kite-app home page notification.
#[derive(sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleNotice {
    /// id
    pub id: i32,
    /// publish time of the notice
    pub publish_time: DateTime<Local>,
    /// Title
    pub title: String,
    /// Content of the notice.
    pub content: Option<String>,
    /// is notice pin top.
    pub top: bool,
}

pub async fn get_recent_notice(client: &PgPool, page: &PageView) -> Result<Vec<SimpleNotice>> {
    let notices = sqlx::query_as(
        "SELECT id, publish_time, title, content, top FROM notice WHERE expired = false
                 ORDER BY top DESC, publish_time DESC
                 LIMIT $1 OFFSET $2;",
    )
    .bind(page.count(20))
    .bind(page.offset(20))
    .fetch_all(client)
    .await?;

    Ok(notices)
}
