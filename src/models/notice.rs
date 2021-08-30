use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;

use crate::error::Result;

/// WeChat Miniprogram home page notification.
#[derive(sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    /// id
    pub id: i32,
    /// publish time of the notice
    pub publish_time: NaiveDateTime,
    /// Title
    pub title: String,
    /// Content of the notice.
    pub content: Option<String>,
    /// is notice pin top.
    pub top: bool,
}

impl Notice {
    pub async fn get(client: &PgPool) -> Result<Vec<Self>> {
        let notices = sqlx::query_as(
            "SELECT id, publish_time, title, content, top FROM notice WHERE expired = false
                 ORDER BY top DESC, publish_time DESC LIMIT 5",
        )
        .fetch_all(client)
        .await?;

        Ok(notices)
    }
}
