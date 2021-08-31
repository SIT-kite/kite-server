//! This module includes interfaces about the event and sign.
use actix_web::{get, post, web, HttpResponse};

use crate::bridge::{
    Activity, ActivityDetail, ActivityDetailRequest, ActivityListRequest, HostError, RequestFrame,
    RequestPayload, ResponsePayload,
};
use crate::error::{ApiError, Result};
use crate::models::event::{Event, EventError};
use crate::models::user::Person;
use crate::models::{event, CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};
use serde_json::json;

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

#[get("/event/sc/activitylist")]
pub async fn query_activity_list(
    app: web::Data<AppState>,
    params: web::Query<ActivityListRequest>,
) -> Result<HttpResponse> {
    let params = params.into_inner();

    let agents = &app.agents;
    let request = RequestFrame::new(RequestPayload::ActivityList(params));
    let response = agents.request(request).await??;
    if let ResponsePayload::ActivityList(list) = response {
        let response = json!({
            "ActivityList": list,
        });
        Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}
