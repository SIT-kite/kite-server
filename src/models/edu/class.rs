use crate::error::Result;
use chrono::{Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryAs, PgPool};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassTime {
    #[serde(rename = "type")]
    pub _type: u8,
    pub week_range: (u8, u8),
    pub day_range: (u8, u8),
    pub day_index: u8,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CourseClass {
    pub term: String,
    pub code: String,
    pub title: String,
    pub css_id: String,
    pub _type: String,
    pub credit: f32,
    pub teacher: Vec<String>,
    pub place: Vec<String>,
    pub campus: String,  // todo: convert to number.
    pub plan_count: i16, // 添加 提升空间字段
    pub selected_count: i16,
    pub arranged_class: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
    pub schedule: serde_json::Value,
}

pub fn get_current_term() -> String {
    let today = chrono::Local::today();
    let (year, month) = (today.year(), today.month() as i32);

    format!("{}{}", year, if (2..7).contains(&month) { "A" } else { "B" })
}

impl CourseClass {
    pub async fn list(pool: &PgPool, course_code: &String, term: &String) -> Result<Vec<Self>> {
        let results: Vec<CourseClass> = sqlx::query_as(
            "SELECT
                    term, code, title, css_id, type AS _type, credit, teacher, place, campus,
                    plan_count, selected_count, arranged_class, note, schedule
                FROM course.list
                WHERE code = $1 AND term = $2",
        )
        .bind(course_code)
        .bind(term)
        .fetch_all(pool)
        .await?;
        Ok(results)
    }
}
