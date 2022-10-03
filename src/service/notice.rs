use poem::web::Data;
use poem_openapi::{
    param::Query,
    payload::Json,
    OpenApi,
};
use sqlx::PgPool;

use crate::model::{notice, PageView};
use crate::response::ApiResponse;
use super::ApiTags;

pub struct NoticeApi;

#[OpenApi]
impl NoticeApi {
    #[oai(path = "/notice", method = "get", tag = "ApiTags::Notice")]
    pub async fn get_notice_list(
        &self,
        pool: Data<&PgPool>,
        index: Query<Option<i32>>,
        count: Query<Option<i32>>,
    ) -> poem::Result<Json<serde_json::Value>> {
        let data = notice::get_recent_notice(
            &pool,
            &PageView {
                index: index.0,
                count: count.0,
            },
        )
        .await?;
        let response: serde_json::Value = ApiResponse::normal(data).into();

        Ok(Json(response))
    }
}
