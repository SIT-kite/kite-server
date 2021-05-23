use chrono::Datelike;
use serde::Serialize;
use sqlx::PgPool;

use crate::error::Result;
use crate::models::PageView;

// Unused.
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ClassTime {
//     /// Class type. 0 for every week, 1 for odd weeks, 2 for even weeks.
//     #[serde(rename = "type")]
//     pub _type: u8,
//     pub week_range: (u8, u8),
//     pub day_range: (u8, u8),
//     pub day_index: u8,
// }

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CourseBase {
    /// Term that the class open.
    pub term: String,
    /// Course code
    pub code: String,
    /// Course name
    pub title: String,
    /// Course type
    #[serde(rename = "type")]
    pub _type: String,
    /// Course credit
    pub credit: f32,
    /// Class count
    pub class_count: i16,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CourseClass {
    /// Class id
    pub class_id: String,
    pub teacher: Vec<String>,
    pub place: Vec<String>,
    pub campus: String,
    // todo: convert to number.
    pub plan_count: i16,
    // 添加 提升空间字段
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

pub fn is_valid_term(term: &str) -> bool {
    let re = regex::Regex::new(r"^20[\d]{2}[AB]$").unwrap();
    re.is_match(term)
}

impl CourseBase {
    pub async fn get(pool: &PgPool, course_code: &str, term: &str) -> Result<Option<Self>> {
        let results: Option<Self> = sqlx::query_as(
            "SELECT DISTINCT 
                    term, list.code, title, type AS _type, credit, CAST(c.class_count AS int2)
                FROM course.list
                INNER JOIN (
                    SELECT code, COUNT(*) AS class_count FROM course.list WHERE term = $2 GROUP BY code
                    ) AS c
                ON c.code = list.code
                WHERE list.code = $1 AND term = $2
                LIMIT 1",
        )
        .bind(course_code)
        .bind(term)
        .fetch_optional(pool)
        .await?;
        Ok(results)
    }

    pub async fn query(
        pool: &PgPool,
        query_string: &str,
        term: &str,
        page: &PageView,
    ) -> Result<Vec<Self>> {
        let results: Vec<Self> = sqlx::query_as(
            "SELECT DISTINCT 
                    term, list.code, title, type AS _type, credit, CAST(c.class_count AS int2) 
                FROM course.list
                INNER JOIN (
                    SELECT code, COUNT(*) AS class_count FROM course.list WHERE term = $2 GROUP BY code
                    ) AS c
                ON c.code = list.code
                WHERE title like $1 AND term = $2
                LIMIT $3 OFFSET $4",
        )
        .bind(format!("%{}%", query_string))
        .bind(term)
        .bind(page.count(20) as i16)
        .bind(page.offset(20) as i16)
        .fetch_all(pool)
        .await?;
        Ok(results)
    }
}

impl CourseClass {
    pub async fn list(pool: &PgPool, course_code: &str, term: &str) -> Result<Vec<Self>> {
        let results: Vec<CourseClass> = sqlx::query_as(
            "SELECT
                    class_id, teacher, place, campus, plan_count, selected_count, arranged_class, note, schedule
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

#[cfg(test)]
mod test {
    #[test]
    pub fn test_term_validator() {
        use super::is_valid_term;

        assert_eq!(true, is_valid_term("2020A"));
        assert_eq!(false, is_valid_term("2020AB"));
        assert_eq!(false, is_valid_term("0019B"));
    }
}
