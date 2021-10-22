//! This module includes interfaces about the event and sign.
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::bridge::{
    ActivityDetailRequest, AgentManager, HostError, RequestFrame, RequestPayload, ResponsePayload,
    SaveScActivity, SaveScScore, ScActivityRequest, ScJoinRequest, ScScoreItemRequest, ScScoreSummary,
};
use crate::error::{ApiError, Result};
use crate::models::edu::{
    get_sc_score_detail, query_activity_detail, query_current_sc_activity_list,
    query_current_sc_score_list, save_sc_activity_detail, save_sc_activity_list, save_sc_score_list,
};
use crate::models::event::{
    get_sc_activity_detail, get_sc_activity_list, query_sc_score, Event, EventError, ScScore,
};
use crate::models::sc::{delete_sc_score_list, save_image, save_image_as_file};
use crate::models::user::Person;
use crate::models::{event, CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};

/**********************************************************************
    Interfaces in this module:
    list_events()         <-- get  /event

    // TODO: implementing.
    create_event()        <-- post /event
    get_event()           <-- get  /event/{event_id}
    get_participants()    <-- get  /event/{event_id}/participant
    participate()         <-- post /event/{event_id}/participant
*********************************************************************/

#[get("/event")]
pub async fn list_events(app: web::Data<AppState>, page: web::Query<PageView>) -> Result<HttpResponse> {
    let parameters: PageView = page.into_inner();

    let event_summaries =
        event::Event::list(&app.pool, parameters.index() as u32, parameters.count(10) as u32).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(event_summaries)))
}

#[post("/event")]
pub async fn create_event(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    form: web::Form<Event>,
) -> Result<HttpResponse> {
    //User need log in before create event
    if token.is_none() {
        return Err(ApiError::new(CommonError::LoginNeeded));
    }

    //User need identity before create event
    let uid = token.unwrap().uid;
    if Person::get_identity(&app.pool, uid).await?.is_none() {
        return Err(ApiError::new(EventError::NeedIdentity));
    }

    let parameter: Event = form.into_inner();
    let mut event: Event = Event::new();

    event.publisher_uid = parameter.publisher_uid;
    event.description = parameter.description;
    event.title = parameter.title;
    event.start_time = parameter.start_time;
    event.end_time = parameter.end_time;
    event.tags = parameter.tags;
    event.place = parameter.place;
    event.image = parameter.image;

    event.create(&app.pool).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::normal(event)))
}

#[derive(Debug, Deserialize)]
pub struct ScDetailQuery {
    pub force: bool,
}

#[get("/event/sc/score")]
pub async fn get_sc_score_list(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    params: web::Query<ScDetailQuery>,
) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;
    let account = identity.student_id;
    let password = identity.oa_secret;
    let params = params.into_inner();

    if params.force {
        let agent = &app.agents;
        let pool = &app.pool;
        loop {
            let score_data = ScScoreItemRequest {
                account: account.clone(),
                passwd: password.clone(),
            };

            let missing_activities: Vec<(i32, i32)> =
                store_sc_score_list(agent, pool, score_data.clone(), account.clone()).await?;
            if missing_activities.is_empty() {
                break;
            }
            // Add missing activity into db
            for missing_activity in missing_activities {
                let (activity_id, category) = missing_activity;
                let activity = ActivityDetailRequest { id: activity_id };
                let mut activity_detail = query_activity_detail(&app.agents, activity).await?;
                activity_detail.category = category;

                let (image_message, image_uuid) = save_image_as_file(&activity_detail.images).await?;

                save_image(&app.pool, image_message).await?;
                save_sc_activity_detail(&app.pool, activity_detail.as_ref(), image_uuid).await?;
            }

            // Delete the score_detail from db
            delete_sc_score_list(&app.pool, account.clone()).await?;

            // Reload db
            store_sc_score_list(agent, pool, score_data, account.clone()).await?;
        }

        let activity_data = ScActivityRequest {
            account: account.clone(),
            passwd: password.clone(),
        };
        let activity_detail = query_current_sc_activity_list(agent, activity_data).await?;
        let save_activity_detail: Vec<SaveScActivity> = activity_detail
            .into_iter()
            .map(|e| SaveScActivity {
                account: account.clone(),
                activity_id: e.activity_id,
                time: e.time,
                status: e.status,
            })
            .collect();
        save_sc_activity_list(&app.pool, save_activity_detail).await?;
    }
    let result = get_sc_score_detail(&app.pool, &account.clone()).await?;
    let response = serde_json::json!({
            "detail": result,
    });
    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
}

async fn store_sc_score_list(
    agent: &AgentManager,
    pool: &PgPool,
    score_data: ScScoreItemRequest,
    account: String,
) -> Result<Vec<(i32, i32)>> {
    let score_detail = query_current_sc_score_list(agent, score_data).await?;
    let save_score_detail: Vec<SaveScScore> = score_detail
        .into_iter()
        .map(|e| SaveScScore {
            account: account.clone(),
            activity_id: e.activity_id,
            category: e.category,
            amount: e.amount,
        })
        .collect();

    let mut missing_activities = Vec::new();
    for score_detail in save_score_detail {
        let db_result = save_sc_score_list(pool, score_detail).await?;
        let (activity_id, category) = db_result.activity_id_category.split_once("-").unwrap_or_default();
        let activity_id: i32 = activity_id.parse().unwrap_or_default();
        let category: i32 = category.parse().unwrap_or_default();
        if activity_id != 0 {
            missing_activities.push((activity_id, category));
        }
    }

    Ok(missing_activities)
}

#[get("/event/sc/score/summary")]
pub async fn get_sc_score(token: Option<JwtToken>, app: web::Data<AppState>) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

    let account = identity.student_id;
    let sc_score = query_sc_score(&app.pool, &account).await?;
    let result = add_score(sc_score);
    let response = serde_json::json!({
        "summary": result,
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

fn add_score(sc_score: Vec<ScScore>) -> ScScoreSummary {
    let mut result = ScScoreSummary {
        total: 0.0,
        theme_report: 0.0,
        social_practice: 0.0,
        creativity: 0.0,
        safety_civilization: 0.0,
        charity: 0.0,
        campus_culture: 0.0,
    };

    for each_score in sc_score {
        match each_score.category {
            1 => {
                result.total += each_score.amount;
                result.theme_report += each_score.amount;
            }
            2 => {
                result.total += each_score.amount;
                result.social_practice += each_score.amount;
            }
            3 => {
                result.total += each_score.amount;
                result.creativity += each_score.amount;
            }
            4 => {
                result.total += each_score.amount;
                result.safety_civilization += each_score.amount;
            }
            5 => {
                result.total += each_score.amount;
                result.charity += each_score.amount;
            }
            6 => {
                result.total += each_score.amount;
                result.campus_culture += each_score.amount;
            }
            7 => {
                result.total += each_score.amount;
                result.theme_report += each_score.amount;
            }
            8 => {
                result.total += each_score.amount;
                result.campus_culture += each_score.amount;
            }
            9 => {
                result.total += each_score.amount;
                result.safety_civilization += each_score.amount;
            }
            _ => {}
        }
    }

    result
}

#[get("/event/sc")]
pub async fn get_sc_event_list(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let _ = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;

    let result = get_sc_activity_list(&app.pool, &page).await?;
    let response = serde_json::json!({
        "activityList": result,
    });

    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
}

#[get("/event/sc/{activity_id}")]
pub async fn get_sc_event_detail(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    activity_id: web::Path<i32>,
) -> Result<HttpResponse> {
    let _ = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;

    let result = get_sc_activity_detail(&app.pool, activity_id.into_inner()).await?;
    let response = serde_json::json!({
        "activityDetail": result,
    });

    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
}

#[derive(Debug, Deserialize)]
pub struct ScActivityApplyQuery {
    pub force: bool,
}

#[post("/event/sc/{activity_id}/apply")]
pub async fn apply_sc_event_activity(
    token: Option<JwtToken>,
    app: web::Data<AppState>,
    activity_id: web::Path<i32>,
    params: web::Query<ScActivityApplyQuery>,
) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;
    let identity = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

    let params = params.into_inner();

    let data = ScJoinRequest {
        account: identity.student_id,
        password: identity.oa_secret,
        activity_id: activity_id.into_inner(),
        force: params.force,
    };

    let agents = &app.agents;
    let request = RequestFrame::new(RequestPayload::ScActivityJoin(data));
    let response = agents.request(request).await??;
    if let ResponsePayload::ScActivityJoin(result) = response {
        let response = json!({
            "result": result,
        });
        Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}
