use chrono::{DateTime, Datelike, Local, NaiveDate};
use poem::web::{Data, Json, Path, Query};
use poem::{handler, Result};
use serde_json::json;
use sqlx::PgPool;
use std::fmt::Debug;

use crate::model::library;
use crate::response::ApiResponse;
use crate::service::jwt::JwtToken;
use tokio::sync::OnceCell;

static PRIVATE_KEY: OnceCell<String> = OnceCell::const_new();
static PUBLIC_KEY: OnceCell<String> = OnceCell::const_new();

async fn load_key(filename: &str) -> String {
    tokio::fs::read_to_string(filename)
        .await
        .expect(&format!("Could not open key file: {}", filename))
}

async fn load_public_key() -> String {
    load_key("./rsa2048-public.pem").await
}

async fn load_private_key() -> String {
    load_key("./rsa2048-private.pem").await
}

fn make_index_description(index: i32) -> &'static str {
    if index == 0 {
        // 无座位
        unreachable!();
    } else if index <= 175 {
        "B201"
    } else {
        "B206"
    }
}

#[handler]
pub async fn get_notice(pool: Data<&PgPool>) -> Result<Json<serde_json::Value>> {
    let data = library::get_notice(&pool).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[derive(serde::Serialize)]
pub struct Status {
    /// 场次
    pub period: i32,
    /// 总允许人数
    pub count: i32,
    /// 已申请人数
    pub applied: i32,
    /// 场次描述
    pub text: String,
    /// 用户是否已经申请
    pub appointed: bool,
}

#[handler]
pub async fn get_status(
    pool: Data<&PgPool>,
    Path(date): Path<i32>,
    token: Option<JwtToken>,
) -> Result<Json<serde_json::Value>> {
    let week_day = NaiveDate::from_ymd(date / 10000, (date / 100 % 100) as u32, (date % 100) as u32)
        .weekday()
        .number_from_monday();
    if week_day == 1 || week_day == 2 {
        let response = ApiResponse::normal(Vec::<Status>::new()).into();
        return Ok(Json(response));
    }

    let status = library::get_status(&pool, date % 1000000).await?;
    let applied_vec = if token.is_some() {
        library::get_applications(&pool, None, None, token.map(|x| x.uid), Some(date % 1000000))
            .await?
            .iter()
            .map(|x| x.period)
            .collect()
    } else {
        vec![]
    };

    fn in_vec<T: std::cmp::PartialEq + Debug>(num: T, vec: &[T]) -> bool {
        for e in vec {
            if e == &num {
                return true;
            }
        }
        return false;
    }
    let make_period_description = |x: i32| match x {
        1 => "9:00 - 11:30",
        2 => "13:30 - 16:00",
        3 => "18:00 - 21:00",
        _ => "/",
    };
    let result: Vec<Status> = status
        .iter()
        .map(|s| Status {
            period: s.period,
            count: 274,
            applied: s.applied,
            text: make_period_description(s.period % 10).to_string(),
            appointed: in_vec(s.period, &applied_vec),
        })
        .collect();
    let response: serde_json::Value = ApiResponse::normal(result).into();
    Ok(Json(response))
}

/// 预约记录查询参数
#[derive(serde::Deserialize)]
pub struct ApplicationQuery {
    pub period: Option<i32>,
    pub user: Option<String>,
    pub date: Option<i32>,
}

#[derive(serde::Serialize)]
pub struct ApplicationResult {
    /// 预约编号
    pub id: i32,
    /// 预约场次. 格式为 `yyMMdd` + 场次 (1 上午, 2 下午, 3 晚上）
    pub period: i32,
    /// 学号/工号
    pub user: String,
    /// 场次下座位号
    pub index: i32,
    /// 预约状态
    pub status: i32,
    /// 房间描述
    pub text: String,
    /// 预约时间
    pub ts: DateTime<Local>,
}

#[handler]
pub async fn get_application_list(
    pool: Data<&PgPool>,
    Query(query): Query<ApplicationQuery>,
    token: JwtToken,
) -> Result<Json<serde_json::Value>> {
    // TODO: 权限校验
    let data: Vec<ApplicationResult> =
        library::get_applications(&pool, query.period, query.user, None, query.date.map(|x| x % 1000000))
            .await?
            .into_iter()
            .map(|e| ApplicationResult {
                id: e.id,
                period: e.period,
                user: e.user,
                index: e.index,
                status: e.status,
                ts: e.ts,
                text: make_index_description(e.index).to_string(),
            })
            .collect();

    let response: serde_json::Value = ApiResponse::normal(data).into();
    Ok(Json(response))
}

#[handler]
pub async fn get_public_key() -> Result<String> {
    let key = PUBLIC_KEY.get_or_init(load_public_key).await;
    Ok(key.clone())
}

#[handler]
pub async fn get_code(pool: Data<&PgPool>, Path(apply_id): Path<i32>) -> Result<Json<serde_json::Value>> {
    let key = PRIVATE_KEY.get_or_init(load_private_key).await;
    let data = library::get_code(&pool, apply_id, key).await?;

    let response: serde_json::Value = ApiResponse::normal(data).into();
    Ok(Json(response))
}

#[derive(serde::Deserialize)]
pub struct ApplyRequest {
    pub period: i32,
}

#[handler]
pub async fn apply(
    pool: Data<&PgPool>,
    token: JwtToken,
    Json(data): Json<ApplyRequest>,
) -> Result<Json<serde_json::Value>> {
    let apply_result = library::apply(&pool, token.uid, data.period).await?;
    let response: serde_json::Value = json!({
        "id": apply_result.id.unwrap(),
        "index": apply_result.index,
        "text": make_index_description(apply_result.index),
        "exist": apply_result.is_exist,
    });
    Ok(Json(ApiResponse::normal(response).into()))
}

#[derive(serde::Deserialize)]
pub struct UpdateRequest {
    pub status: i32,
}

#[handler]
pub async fn update_application_status(
    pool: Data<&PgPool>,
    Path(apply_id): Path<i32>,
    Json(data): Json<UpdateRequest>,
    token: JwtToken,
) -> Result<Json<serde_json::Value>> {
    library::update_application(&pool, apply_id, data.status).await?;

    let response: serde_json::Value = ApiResponse::<()>::empty().into();
    Ok(Json(response))
}

#[handler]
pub async fn cancel(
    pool: Data<&PgPool>,
    Path(apply_id): Path<i32>,
    token: JwtToken,
) -> Result<Json<serde_json::Value>> {
    // TODO: Check token.
    library::cancel(&pool, apply_id).await?;

    let response: serde_json::Value = ApiResponse::<()>::empty().into();
    Ok(Json(response))
}

#[handler]
pub async fn get_current_period() -> Result<Json<serde_json::Value>> {
    let now = Local::now();
    let period = library::make_period_by_datetime(now);
    let response = match period {
        Some(period) => {
            let range = library::get_period_range(now.date(), period % 10);
            json!({
                "period": period,
                "after": range.0,
                "before": range.1,
            })
        }
        None => {
            let next = library::get_next_period(now);
            json!({
                "next":  next,
                "period": Option::<()>::None,
            })
        }
    };
    Ok(Json(ApiResponse::normal(response).into()))
}
