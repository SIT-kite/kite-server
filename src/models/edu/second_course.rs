use sqlx::PgPool;

use crate::bridge::{
    AgentManager, HostError, RequestFrame, RequestPayload, ResponsePayload, SaveScScore, ScScoreItem,
    ScScoreItemRequest,
};
use crate::error::{ApiError, Result};

pub async fn query_current_sc_score_list(
    agent: &AgentManager,
    data: ScScoreItemRequest,
) -> Result<Vec<ScScoreItem>> {
    let request = RequestFrame::new(RequestPayload::ScScoreDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ScScoreDetail(scscore) = response {
        Ok(scscore)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_sc_score_list(db: &PgPool, data: Vec<SaveScScore>) -> Result<()> {
    for each_score in data {
        sqlx::query(
            "INSERT INTO edu.sc_score_detail (student_id, activity_id, amount)
            VALUES ($1, $2, $3);",
        )
        .bind(each_score.account)
        .bind(each_score.activity_id)
        .bind(each_score.amount)
        .execute(db)
        .await?;
    }
    Ok(())
}
