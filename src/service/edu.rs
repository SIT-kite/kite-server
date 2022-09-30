use std::time::Duration;

use poem::web::Json;
use poem::{handler, Result};
use tokio::io::AsyncWriteExt;

use crate::config::CONFIG;
use crate::response::ApiResponse;
use crate::service::JwtToken;

#[handler]
pub async fn upload_temporary_calendar(text: String, _token: JwtToken) -> Result<Json<serde_json::Value>> {
    let uuid = uuid::Uuid::new_v4();
    let path = format!("{}/temp/{}.ics", &CONFIG.get().unwrap().attachment, uuid);
    let mut file = tokio::fs::File::create(&path).await.unwrap();

    file.write_all(text.as_ref()).await;
    let response = serde_json::json!({
        "url": format!("https://kite.sunnysab.cn/static/temp/{}.ics", uuid),
        "timeout": 300,
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5 * 60)).await;
        tokio::fs::remove_file(path).await;
    });
    Ok(Json(ApiResponse::normal(response).into()))
}
