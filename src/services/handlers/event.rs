//! This module includes interfaces about the event and sign.
use crate::error::Result;
use crate::models::event;
use crate::services::NormalResponse;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

/**********************************************************************
    Interfaces in this module:
    list_events()         <-- get  /event

    // TODO: implementing.
    create_event()        <-- post /event
    get_event()           <-- get  /event/{event_id}
    get_participants()    <-- get  /event/{event_id}/participant
    participate()         <-- post /event/{event_id}/participant
*********************************************************************/

#[derive(Debug, Deserialize)]
pub struct ListEvent {
    #[serde(rename = "pageIndex")]
    page_index: Option<u32>,
    count: Option<u32>,
}

#[get("/event")]
pub async fn list_events(pool: web::Data<PgPool>, form: web::Query<ListEvent>) -> Result<HttpResponse> {
    let parameters: ListEvent = form.into_inner();
    let event_summaries = event::Event::list(
        &pool,
        parameters.page_index.unwrap_or(1),
        parameters.count.unwrap_or(10),
    )
    .await?;

    Ok(HttpResponse::Ok().json(&NormalResponse::new(event_summaries)))
}
