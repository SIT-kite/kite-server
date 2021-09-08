use sqlx::PgPool;

use crate::bridge::{
    Activity, ActivityDetail, ActivityDetailRequest, ActivityListRequest, AgentManager, HostError,
    RequestFrame, RequestPayload, ResponsePayload, SaveScActivity, SaveScScore, ScActivityItem,
    ScActivityRequest, ScDetail, ScScoreItem, ScScoreItemRequest,
};
use crate::error::{ApiError, Result};
use crate::services::AppState;
use actix_web::web;

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
        "SELECT detail.activity_id, event.title, time, status, amount 
        FROM events.sc_detail as detail, events.sc_events as event
        WHERE detail.activity_id = event.activity_id and student_id = $1;",
    )
    .bind(query)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn query_activity_list(
    agent: &AgentManager,
    data: ActivityListRequest,
) -> Result<Vec<Activity>> {
    let request = RequestFrame::new(RequestPayload::ActivityList(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ActivityList(list) = response {
        Ok(list)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn query_activity_detail(
    agent: &AgentManager,
    data: ActivityDetailRequest,
) -> Result<Box<ActivityDetail>> {
    let request = RequestFrame::new(RequestPayload::ActivityDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ActivityDetail(detail) = response {
        Ok(detail)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_sc_activity_detail(db: &PgPool, data: Box<ActivityDetail>) -> Result<()> {
    sqlx::query(
        "INSERT INTO events.sc_events (activity_id, category, title, start_time, sign_time, end_time, place, duration, manager, contact, organizer, undertaker, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (activity_id) DO NOTHING;"
    )
        .bind(data.id)
        .bind(data.category)
        .bind(data.title)
        .bind(data.start_time)
        .bind(data.sign_time)
        .bind(data.end_time)
        .bind(data.place)
        .bind(data.duration)
        .bind(data.manager)
        .bind(data.contact)
        .bind(data.organizer)
        .bind(data.undertaker)
        .bind(data.description)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_and_save_sc_activity_detail(
    app: web::Data<AppState>,
    params: web::Query<ActivityListRequest>,
) -> Result<()> {
    let params = params.into_inner();
    let agent = &app.agents;
    let activity_list = query_activity_list(agent, params).await?;
    for each_activity in activity_list {
        let data = ActivityDetailRequest { id: each_activity.id };
        let mut activity_detail = query_activity_detail(agent, data).await?;
        activity_detail.category = each_activity.category;
        save_sc_activity_detail(&app.pool, activity_detail).await?;
    }
    Ok(())
}
