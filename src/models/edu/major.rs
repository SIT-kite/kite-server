use crate::error::Result;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;

/// Correspondence between the school's professional codes and names
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Major {
    /// Major category
    pub category: String,
    /// Major code used in ems.sit.edu.cn
    pub code: String,
    /// Major name.
    pub title: String,
    /// Last update time of planned course.
    pub last_update: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PlannedCourse {
    /* Comment on 2020.7.29
       Because query condition set unique major and year, both items in this structure
       is not necessary.
    */
    // pub major: String,
    // pub year: i16,
    /// Course category, like 专业基础课
    pub course_category: String,
    /// Course code in system.
    pub code: String,
    /// Course name
    pub title: String,
    /// Some professional courses and public courses may not have
    /// formal exams, but quizzes or essays instead.
    pub has_test: bool,
    /// Course credit
    pub credit: f32,
    /// The college belong to
    // pub department: String,
    /// You will get this course in this term normally.
    /// If the value is None, maybe it's uncertain.
    pub term: Option<i16>,
}

impl Major {
    /// Query majors title or code.
    pub async fn query(pool: &PgPool, query_string: &str) -> Result<Vec<Self>> {
        if query_string.is_empty() {
            return Ok(vec![]);
        }
        let results: Vec<Major> = sqlx::query_as(
            "SELECT category, code, title, last_update
                FROM course.majors
                INNER JOIN (
                    SELECT major, COUNT(*) AS count
                    FROM course.major_plan
                    GROUP BY major
                    HAVING count(*) > 0
                    ) s
                ON s.major = majors.code
                WHERE (title LIKE $1 AND code is not null)
                   OR  majors.category =
                   (SELECT category FROM course.majors WHERE title like $1 AND code is null LIMIT 1)
                ORDER BY code",
        )
        .bind(format!("%{}%", query_string))
        .fetch_all(pool)
        .await?;
        Ok(results)
    }
}

impl PlannedCourse {
    /// Query planned courses of certain majors.
    pub async fn query(pool: &PgPool, major_code: &str, enter_year: i16) -> Result<Vec<Self>> {
        let results: Vec<PlannedCourse> = sqlx::query_as(
            "SELECT c.title AS course_category, mp.code, mp.title, has_test, credit, term
                FROM course.major_plan mp
                INNER JOIN course.category c
                ON mp.course_category = c.code
                WHERE major = $1 AND year = $2
                ORDER BY mp.term;",
        )
        .bind(major_code)
        .bind(enter_year)
        .fetch_all(pool)
        .await?;
        Ok(results)
    }
}
