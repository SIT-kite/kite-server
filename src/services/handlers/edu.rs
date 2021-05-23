//! This module includes interfaces about course, major and score.

use actix_web::{get, web, HttpResponse};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::edu::{self, CourseBase, CourseClass, Major, PlannedCourse};
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[derive(Debug, Deserialize)]
pub struct ListMajor {
    pub q: String,
}

#[get("/edu/major")]
pub async fn query_major(
    app: web::Data<AppState>,
    query: web::Query<ListMajor>,
) -> Result<HttpResponse> {
    let parameters: ListMajor = query.into_inner();
    let results = Major::query(&app.pool, &parameters.q).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(results)))
}

#[derive(Debug, Deserialize)]
pub struct ListPlannedCourse {
    pub year: Option<i16>,
}

#[get("/edu/major/{major_code}")]
pub async fn get_planned_course(
    app: web::Data<AppState>,
    major_code: web::Path<String>,
    query: web::Query<ListPlannedCourse>,
) -> Result<HttpResponse> {
    use chrono::Local;

    // Get local current year as the default year.
    let mut year = Local::today().naive_local().year() as i16;
    // If user submit a year and it's valid, then use user defined year.
    // This limit is to reduce possible unnecessary database queries
    if let Some(input_year) = query.into_inner().year {
        if (2017..=year).contains(&input_year) {
            year = input_year;
        }
    }
    let results = PlannedCourse::query(&app.pool, &major_code, year).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(results)))
}

#[derive(Debug, Deserialize)]
pub struct ListClasses {
    pub term: Option<String>,
}

#[get("/edu/course/{course_code}")]
pub async fn list_course_classes(
    app: web::Data<AppState>,
    course_code: web::Path<String>,
    query: web::Query<ListClasses>,
) -> Result<HttpResponse> {
    let term = query.into_inner().term;
    // If term field exists, check it.
    if let Some(term) = &term {
        if !edu::is_valid_term(term.as_str()) {
            return Err(CommonError::Parameter.into());
        }
    }
    let term_string = term.unwrap_or_else(edu::get_current_term);
    let course = CourseBase::get(&app.pool, &course_code, &term_string).await?;
    if let Some(course) = course {
        let classes = CourseClass::list(&app.pool, &course_code, &term_string).await?;
        #[derive(Serialize)]
        struct Response {
            pub course: CourseBase,
            pub classes: Vec<CourseClass>,
        }
        Ok(HttpResponse::Ok().json(&ApiResponse::normal(Response { course, classes })))
    } else {
        Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchCourse {
    pub q: String,
    pub term: Option<String>,
}

#[get("/edu/course")]
pub async fn query_course(
    app: web::Data<AppState>,
    query: web::Query<SearchCourse>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let parameters = query.into_inner();
    let term = parameters.term;
    // If term field exists, check it.
    if let Some(term) = &term {
        if !edu::is_valid_term(term.as_str()) {
            return Err(CommonError::Parameter.into());
        }
    }
    if parameters.q.is_empty() {
        return Ok(HttpResponse::Ok().json(&ApiResponse::empty()));
    }
    let term_string = term.unwrap_or_else(edu::get_current_term);
    let course = CourseBase::query(&app.pool, &parameters.q, &term_string, &page).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(course)))
}
