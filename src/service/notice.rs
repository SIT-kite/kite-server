use std::any::Any;
use std::collections::HashMap;

use chrono::{Local, DateTime};
use poem::web::Data;
use poem_openapi::{
    param::Query,
    payload::Json,
    OpenApi,
};

use sqlx::PgPool;
use tokio::fs::File;

use crate::response::ApiResponse;
use super::ApiTags;
use crate::config::CONFIG;


pub struct NoticeApi;

/// Kite-app home page notification.
#[derive(serde::Serialize, serde::Deserialize)]
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

#[OpenApi]
impl NoticeApi {
    #[oai(path = "/notice", method = "get", tag = "ApiTags::Notice")]
    pub async fn get_notice_list(&self) -> poem::Result<Json<serde_json::Value>> {
        let mut path = CONFIG.get().unwrap().static_api_dir.clone();
        path += "/bulletin.json";
        let content = tokio::fs::read_to_string(path).await.unwrap();
        let data = serde_json::from_str::<Vec<SimpleNotice>>(&content).unwrap();
        let response: serde_json::Value = ApiResponse::normal(data).into();
        Ok(Json(response))
    }
}
