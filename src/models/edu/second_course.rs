use sqlx::PgPool;

use crate::bridge::{
    AgentManager, HostError, RequestFrame, RequestPayload, ResponsePayload, SaveScActivity, SaveScScore,
    ScActivityItem, ScActivityRequest, ScDetail, ScScoreItem, ScScoreItemRequest,
};
use crate::error::{ApiError, Result};

pub async fn query_current_sc_score_list(
    agent: &AgentManager,
    data: ScScoreItemRequest,
) -> Result<Vec<ScScoreItem>> {
    let request = RequestFrame::new(RequestPayload::ScScoreDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ScScoreDetail(sc_score) = response {
        Ok(sc_score)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_sc_score_list(db: &PgPool, data: Vec<SaveScScore>) -> Result<()> {
    for each_score in data {
        sqlx::query(
            "INSERT INTO events.sc_score_detail (student_id, activity_id, amount)
            VALUES ($1, $2, $3)
            ON CONFLICT (student_id, activity_id, amount) DO NOTHING;",
        )
        .bind(each_score.account)
        .bind(each_score.activity_id)
        .bind(each_score.amount)
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn query_current_sc_activity_list(
    agent: &AgentManager,
    data: ScActivityRequest,
) -> Result<Vec<ScActivityItem>> {
    let request = RequestFrame::new(RequestPayload::ScActivityDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ScActivityDetail(sc_activity) = response {
        Ok(sc_activity)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_sc_activity_list(db: &PgPool, data: Vec<SaveScActivity>) -> Result<()> {
    for each_activity in data {
        sqlx::query(
            "INSERT INTO events.sc_activity_detail (student_id, activity_id, time, status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (student_id, activity_id, time) DO NOTHING;",
        )
        .bind(each_activity.account)
        .bind(each_activity.activity_id)
        .bind(each_activity.time)
        .bind(each_activity.status)
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn get_sc_score_detail(pool: &PgPool, query: &str) -> Result<Vec<ScDetail>> {
    let result = sqlx::query_as(
        "select activity_id, time, status, amount from events.sc_detail
        where student_id = $1;",
    )
    .bind(query)
    .fetch_all(pool)
    .await?;

    Ok(result)
}
