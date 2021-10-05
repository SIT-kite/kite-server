//! This module includes interfaces about the event and sign.
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use crate::bridge::{
    SaveScActivity, SaveScScore, ScActivityRequest, ScScoreItemRequest, ScScoreSummary,
};
use crate::error::{ApiError, Result};
use crate::models::edu::{
    get_sc_score_detail, query_current_sc_activity_list, query_current_sc_score_list,
    save_sc_activity_list, save_sc_score_list,
};
use crate::models::event::{
    get_sc_activity_detail, get_sc_activity_list, query_sc_score, Event, EventError, ScScore,
};
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

#[get("/event/sc/score_detail")]
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
        let score_data = ScScoreItemRequest {
            account: account.clone(),
            passwd: password.clone(),
        };

        let agent = &app.agents;
        let score_detail = query_current_sc_score_list(agent, score_data).await?;
        let save_score_detail: Vec<SaveScScore> = score_detail
            .into_iter()
            .map(|e| SaveScScore {
                account: account.clone(),
                activity_id: e.activity_id,
                amount: e.amount,
            })
            .collect();
        save_sc_score_list(&app.pool, save_score_detail).await?;

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

#[get("/event/sc/score")]
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
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;

    let _ = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

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
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let uid = token
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))
        .map(|token| token.uid)?;

    let _ = Person::get_identity(&app.pool, uid)
        .await?
        .ok_or_else(|| ApiError::new(CommonError::IdentityNeeded))?;

    let activity_id = path.into_inner();
    let result = get_sc_activity_detail(&app.pool, activity_id).await?;
    let response = serde_json::json!({
        "activityDetail": result,
    });

    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
}
