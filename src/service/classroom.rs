use crate::error::ApiError;
use crate::model::classroom;
use crate::model::classroom::AvailClassroomQuery;
use crate::response::ApiResponse;
use poem::web::{Data, Json, Query};
use poem::{handler, Result};
use sqlx::PgPool;

use crate::model::{CAMPUS_FENGXIAN, CAMPUS_XUHUI};

#[derive(serde::Deserialize)]
pub struct ClassroomQuery {
    /// 建筑名称, 如 `一教`
    pub building: Option<String>,
    /// 区域名称, 如 A, B, C, D
    pub region: Option<String>,
    /// 校区编号.
    pub campus: Option<i32>,
    /// 要查询的日期, 格式如 `2020-1-1`
    pub date: String,
    /// 期望有空的时间, 如 `1-2,5-6`
    pub time: Option<String>,
}

#[handler]
pub async fn query_available_classrooms(
    pool: Data<&PgPool>,
    Query(query): Query<ClassroomQuery>,
) -> Result<Json<serde_json::Value>> {
    let want_time = query.time.unwrap_or_else(|| String::from("0-0"));
    // See: model::edu::classroom::AvailClassroomQuery::want_time
    let want_time_bits = classroom::convert_time_string(&want_time);

    let campus = query.campus.unwrap_or(0);
    // Judge the campus weather it is true number
    if campus != CAMPUS_FENGXIAN && campus != CAMPUS_XUHUI {
        return Err(ApiError::custom(1, "参数错误").into());
    }

    let (term_week, week_day) = classroom::transform_date(&query.date);
    let query = AvailClassroomQuery {
        building: query.building,
        region: query.region,
        campus: Some(campus),
        week: term_week,
        day: week_day,
        want_time: Some(want_time_bits),
    };
    let data = classroom::query_avail_classroom(&pool, &query).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}
