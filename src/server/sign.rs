//! This module includes interfaces about the event and sign.
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use chrono::{NaiveDateTime, Utc};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ServerError, UserError};
use crate::jwt::{encode_jwt, JwtClaims};
use crate::server::get_uid_by_req;
use crate::sign;
use crate::sign::{ApplicantResult, Event, EventSummary};

use super::NormalResponse;

/*
    Interfaces in this module:
    list_events()         <-- get  /event
    create_event()        <-- post /event
    get_event()           <-- get  /event/{event_id}
    get_participants()    <-- get  /event/{event_id}/participant
    participate()         <-- post /event/{event_id}/participant
*/

pub type Pool<T> = diesel::r2d2::Pool<ConnectionManager<T>>;
pub type PostgresPool = Pool<PgConnection>;

#[derive(Debug, Deserialize)]
pub struct EventRequirement {
    #[serde(rename = "pageSize")]
    page_size: Option<u32>,
    index: Option<u32>,
    filter: Option<String>,
    all: Option<bool>,
}

#[get("/event")]
pub async fn list_events(
    pool: web::Data<PostgresPool>,
    form: web::Form<EventRequirement>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    let parameters: EventRequirement = form.into_inner();

    let event_summaries = sign::list_events(
        &conn,
        parameters.index.unwrap_or(0),
        parameters.page_size.unwrap_or(20),
        parameters.filter,
        parameters.all.unwrap_or(false),
    )?;

    Ok(HttpResponse::Ok().json(&NormalResponse::new(event_summaries)))
}

#[derive(Debug, Deserialize)]
pub struct EventCreateRequest {
    pub title: String,
    pub description: String,
    #[serde(rename = "startTime")]
    pub start_time: Option<u64>,
    #[serde(rename = "endTime")]
    pub end_time: Option<u64>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "maxPeople")]
    pub max_people: Option<i16>,
    pub place: String,
    pub image: i32,
    pub attachments: Option<Vec<i32>>,
}

#[post("/event")]
pub async fn create_event(
    pool: web::Data<PostgresPool>,
    form: web::Form<EventCreateRequest>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    let parameters: EventCreateRequest = form.into_inner();

    let current_timestamp = NaiveDateTime::from_timestamp(
        parameters
            .start_time
            .unwrap_or(Utc::now().timestamp() as u64) as i64,
        0,
    );
    let new_event = Event {
        event_id: 0,
        publisher_uid: 0,
        title: parameters.title,
        description: parameters.description,
        start_time: current_timestamp,
        end_time: if let Some(ts) = parameters.end_time {
            Some(NaiveDateTime::from_timestamp(ts as i64, 0))
        } else {
            None
        },
        create_time: current_timestamp,
        tags: parameters.tags,
        deleted: false,
        max_people: None,
        place: parameters.place,
        image: "".to_string(), // TODO: Get Media Url.
        attachments: parameters.attachments,
    };
    let new_event_id = sign::create(&conn, &new_event)?;

    #[derive(Serialize)]
    struct EventCreationResponse {
        #[serde(rename = "eventId")]
        event_id: i32,
    }
    let resp = EventCreationResponse {
        event_id: new_event_id,
    };
    Ok(HttpResponse::Ok().json(&resp))
}

#[get("/event/{event_id}")]
pub async fn get_event(
    pool: web::Data<PostgresPool>,
    event_id: web::Path<i32>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;

    let event: Event = sign::get_event(&conn, event_id.into_inner())?;
    Ok(HttpResponse::Ok().json(&NormalResponse::new(event)))
}

#[derive(Deserialize)]
pub struct ParticipantPageParameters {
    pub index: Option<i32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i32>,
}

#[get("/event/{event_id}/participant}")]
pub async fn get_participants(
    pool: web::Data<PostgresPool>,
    event_id: web::Path<i32>,
    parameters: web::Form<ParticipantPageParameters>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;

    let event: Vec<ApplicantResult> = sign::get_applicants(
        &conn,
        event_id.into_inner(),
        parameters.index.unwrap_or(0),
        parameters.page_size.unwrap_or(20),
    )?;
    Ok(HttpResponse::Ok().json(&NormalResponse::new(event)))
}

#[post("/event/{event_id}/participant}")]
pub async fn participate(
    pool: web::Data<PostgresPool>,
    event_id: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    let uid = get_uid_by_req(&req);
    let event: Vec<ApplicantResult>;

    // TODO: 精简代码
    if let Some(uid) = uid {
        sign::register(&conn, event_id.into_inner(), uid)?;
        return Ok(HttpResponse::Ok().json(&NormalResponse::new("添加成功".to_string())));
    }
    Err(ServerError::from(UserError::Forbidden))
}
