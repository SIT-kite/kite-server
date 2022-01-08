use poem::handler;
use poem::web::{Data, Json};
use sqlx::PgPool;

use crate::model::notice;
use crate::response::ApiResponse;

#[handler]
pub async fn get_notice_list(pool: Data<&PgPool>) -> poem::Result<Json<serde_json::Value>> {
    let data = notice::get_recent_notice(&pool).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}
