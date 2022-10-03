
use poem::web::Data;
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};
use sqlx::PgPool;

use crate::model::report;
use crate::model::report::UserEvent;
use crate::response::ApiResponse;

use super::ApiTags;

#[derive(serde::Deserialize)]
pub struct UserEventBody {
    /// 用户 UUID
    pub user: uuid::Uuid,
    /// 事件列表
    pub events: Vec<UserEvent>,
}

pub struct ReportApi;

#[OpenApi]
impl ReportApi {
    #[oai(path = "/report/exception", method = "post", tag = "ApiTags::Report")]
    pub async fn post_exception(
        &self,
        pool: Data<&PgPool>,
        exception: Json<serde_json::Value>,
    ) -> poem::Result<Json<serde_json::Value>> {
        let exception_value = serde_json::from_value::<report::Exception>(exception.0).unwrap();
        report::save_exception(&pool,&exception_value).await?;
        Ok(Json(ApiResponse::<()>::empty().into()))
    }
    
    #[oai(path = "/report/event", method = "post", tag = "ApiTags::Report")]
    pub async fn post_user_event(
        &self,
        pool: Data<&PgPool>,
        body: Json<serde_json::Value>,
    ) -> poem::Result<Json<serde_json::Value>> {
        let user_event = serde_json::from_value::<UserEventBody>(body.0).unwrap();
        report::append_user_event(&pool, &user_event.user, &user_event.events).await?;
        let response: serde_json::Value = ApiResponse::<()>::empty().into();
        Ok(Json(response))
    }
    
}
