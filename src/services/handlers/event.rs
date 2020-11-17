//! This module includes interfaces about the event and sign.
use crate::error::Result;
use crate::models::{event, PageView};
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

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
    refresh: Option<bool>,
}

#[get("/event")]
pub async fn list_events(
    app: web::Data<AppState>,
    page: web::Query<PageView>,
    form: web::Query<ListEvent>,
) -> Result<HttpResponse> {
    let parameters: PageView = page.into_inner();

    let event_summaries =
        event::Event::list(&app.pool, parameters.index() as u32, parameters.count(10) as u32).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(event_summaries)))
}
