use poem::handler;
use poem::web::{Data, Json};
use sqlx::PgPool;

use crate::model::report;
use crate::model::report::UserEvent;
use crate::response::ApiResponse;

#[derive(serde::Deserialize)]
pub struct UserEventBody {
    /// 用户 UUID
    pub user: uuid::Uuid,
    /// 事件列表
    pub events: Vec<UserEvent>,
}

#[handler]
pub async fn get_notice_list(
    pool: Data<&PgPool>,
    Json(body): Json<UserEventBody>,
) -> poem::Result<Json<serde_json::Value>> {
    report::append_user_event(&pool, &body.user, &body.events).await?;

    let response: serde_json::Value = ApiResponse::<()>::empty().into();
    Ok(Json(response))
}
