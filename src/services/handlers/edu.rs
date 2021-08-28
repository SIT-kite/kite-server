//! This module includes interfaces about course, major and score.

use actix_web::{get, web, HttpResponse};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::bridge;
use crate::bridge::{Course, RequestPayload, SchoolYear, Semester, TimeTableRequest};
use crate::error::{ApiError, Result};
use crate::models::edu::{self, AvailClassroomQuery, CourseBase, CourseClass, Major, PlannedCourse};
use crate::models::user::Person;
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};

const CAMPUS_XUHUI: i32 = 2;
const CAMPUS_FENGXIAN: i32 = 1;

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

#[derive(serde::Deserialize, sqlx::FromRow)]
pub struct ClassroomQuery {
    pub building: Option<String>,
    pub region: Option<String>,
    pub campus: Option<i32>,
    pub date: String,
    pub time: Option<String>,
}

#[get("/edu/classroom/available")]
pub async fn query_available_classrooms(
    app: web::Data<AppState>,
    query: web::Query<ClassroomQuery>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let query = query.into_inner();

    let want_time = query.time.unwrap_or_else(|| String::from("0-0"));
    // See: model::edu::classroom::AvailClassroomQuery::want_time
    let want_time_bits = edu::convert_time_string(&want_time);

    let campus = query.campus.unwrap_or(0);
    // Judge the campus weather it is true number
    if campus != CAMPUS_FENGXIAN && campus != CAMPUS_XUHUI {
        return Err(ApiError::new(CommonError::Parameter));
    }

    let (term_week, week_day) = edu::transform_date(&query.date);
    let query = AvailClassroomQuery {
        building: query.building,
        region: query.region,
        campus: Some(campus),
        week: term_week,
        day: week_day,
        want_time: Some(want_time_bits),
    };
    let result = edu::query_avail_classroom(&app.pool, &query, &page).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}

#[derive(Debug, Deserialize)]
pub struct TimeTableQuery {
    pub school_year: i32,
    pub semester: i32,
}

#[get("/edu/timetable/student/{student_id}")]
pub async fn timetable_agent(
    student_id: web::Path<String>,
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    params: web::Query<TimeTableQuery>,
) -> Result<HttpResponse> {
    let params = params.into_inner();
    let year = SchoolYear::SomeYear(params.school_year);

    let semester = match params.semester {
        1 => Semester::FirstTerm,
        2 => Semester::SecondTerm,
        3 => Semester::MidTerm,
        _ => Semester::All,
    };
    if token.is_none() {
        return Err(ApiError::new(CommonError::LoginNeeded));
    }
    let uid = token.unwrap().uid;

    let passwd = Person::get_identity(&app.pool, uid).await?;
    if passwd.is_none() {
        return Err(ApiError::new(CommonError::NeedIdentity));
    }
    let password = passwd.unwrap().oa_secret;

    let data = TimeTableRequest {
        account: student_id.to_string(),
        passwd: password,
        school_year: year,
        semester,
    };

    let agents = &app.agents;
    let payload = RequestPayload::TimeTable(data);
    let request = bridge::RequestFrame::new(payload);

    let response = agents.request(request).await?;
    match response {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::normal(response))),
        Err(e) => Err(e.into()),
    }
}
