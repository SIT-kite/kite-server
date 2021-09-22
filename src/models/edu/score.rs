use crate::bridge::{
    trans_year_to_i32, AgentManager, HostError, RequestFrame, RequestPayload, ResponsePayload,
    SaveScore, Score, ScoreDetail, ScoreDetailRequest, ScoreRequest,
};
use crate::error::{ApiError, Result};
use crate::models::CommonError;
use serde_json::Value;
use sqlx::PgPool;

pub async fn get_score(agent: &AgentManager, data: ScoreRequest) -> Result<Vec<Score>> {
    let request = RequestFrame::new(RequestPayload::Score(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::Score(score) = response {
        Ok(score)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn get_score_detail(
    agent: &AgentManager,
    data: ScoreDetailRequest,
) -> Result<Vec<ScoreDetail>> {
    let request = RequestFrame::new(RequestPayload::ScoreDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ScoreDetail(score_detail) = response {
        Ok(score_detail)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_score(db: &PgPool, account: String, data: Score) -> Result<()> {
    let first_year = trans_year_to_i32(data.school_year)?;

    let mut evaluate = true;
    if data.score < 0.0 {
        evaluate = false;
    }

    sqlx::query(
        "INSERT INTO edu.score (student_id, score, course, course_id, class_id, school_year, semester, credit, is_evaluated)
        VALUES ($1, $2, $3, $4 , $5, $6, $7, $8, $9)
        ON CONFLICT (student_id, course_id, school_year, semester) DO UPDATE SET score = $2, is_evaluated = $9;",
    )
        .bind(account)
        .bind(data.score)
        .bind(data.course)
        .bind(data.course_id)
        .bind(data.class_id)
        .bind(first_year)
        .bind(data.semester)
        .bind(data.credit)
        .bind(evaluate)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn save_detail(db: &PgPool, data: Value, class_id: String) -> Result<()> {
    sqlx::query(
        "UPDATE edu.score SET detail = $1
        WHERE class_id = $2;",
    )
    .bind(data)
    .bind(class_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_save_score(pool: &PgPool, account: String, year: String) -> Result<Vec<SaveScore>> {
    let result = sqlx::query_as(
        "SELECT score.score, course, course_id, class_id, school_year, semester, credit, detail, is_evaluated
        FROM edu.score
        WHERE student_id = $1 AND school_year = $2;",
    )
    .bind(account)
    .bind(year)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

#[test]

fn test_json() {
    let x = ScoreDetail {
        score_type: "平时".to_string(),
        percentage: "50%".to_string(),
        score: 93.0,
    };
    let y = ScoreDetail {
        score_type: "期末".to_string(),
        percentage: "50%".to_string(),
        score: 84.0,
    };
    let z = ScoreDetail {
        score_type: "总评".to_string(),
        percentage: "0%".to_string(),
        score: 89.0,
    };

    let mut v = Vec::new();
    v.push(x);
    v.push(y);
    v.push(z);
    use serde_json::json;
    let json = json!(v);
    // let json = serde_json::to_value(v);
    println!("{:?}", json);
}
