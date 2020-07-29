//! This module includes interfaces about course, major and score.

use crate::error::Result;
use crate::models::edu::{Major, PlannedCourse};
use crate::models::CommonError;
use crate::services::response::ApiResponse;
use actix_web::{get, web, HttpResponse};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct ListMajor {
    pub q: String,
}

#[get("/edu/major")]
pub async fn query_major(pool: web::Data<PgPool>, query: web::Query<ListMajor>) -> Result<HttpResponse> {
    let parameters: ListMajor = query.into_inner();
    let results = Major::query(&pool, &parameters.q).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(results)))
}

#[derive(Debug, Deserialize)]
pub struct ListPlannedCourse {
    pub year: i16,
}

#[get("/edu/major/{major_code}")]
pub async fn get_planned_course(
    pool: web::Data<PgPool>,
    major_code: web::Path<String>,
    query: web::Query<ListPlannedCourse>,
) -> Result<HttpResponse> {
    use chrono::Local;

    let parameters: ListPlannedCourse = query.into_inner();
    if parameters.year < 2015 || parameters.year as i32 > Local::today().naive_local().year() {
        return Err(CommonError::Parameter.into());
    }
    let results = PlannedCourse::query(&pool, &major_code, parameters.year).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(results)))
}
