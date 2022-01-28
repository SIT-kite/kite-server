use poem::handler;
use poem::web::{Data, Json};
use sqlx::PgPool;

use crate::model::exception;
use crate::response::ApiResponse;

#[handler]
pub async fn post_exception(
    pool: Data<&PgPool>,
    Json(exception): Json<serde_json::Value>,
) -> poem::Result<Json<serde_json::Value>> {
    exception::save_exception(&pool, &exception).await?;
    Ok(Json(ApiResponse::<()>::empty().into()))
}
