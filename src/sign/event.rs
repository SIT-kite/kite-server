use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::error::{EventError, Result, ServerError};

use super::*;

pub fn create(client: &PgConnection, new_event: &Event) -> Result<i32> {
    use self::schema::events::dsl::*;

    let new_event_id = diesel::insert_into(events)
        .values(new_event)
        .returning(event_id)
        .get_result::<i32>(client)?;
    Ok(new_event_id)
}


pub fn list_events(client: &PgConnection, page_index: u32, page_size: u32) -> Result<Vec<EventSummary>> {
    use self::schema::events::dsl::*;

    let event_records: Vec<Event> = events.order(id.desc())
        .offset(((page_index - 1) * page_size) as i64)
        .limit(page_size as i64)
        .get_results::<Event>(client)?;
    // TODO: Add publish time for events.
    Ok(event_records.into_iter().map(|event| event.summarize()).rev().collect())
}

pub fn list_all_events(client: &PgConnection, page_index: u32, page_size: u32) -> Result<Vec<EventSummary>> {
    use self::schema::events::dsl::*;

    let event_records: Vec<Event> = events.order(id.desc())
        .filter(deleted.eq(false))
        .offset(((page_index - 1) * page_size) as i64)
        .limit(page_size as i64)
        .get_results::<Event>(client)?;
    // TODO: Add publish time for events.
    Ok(event_records.into_iter().map(|event| event.summarize()).rev().collect())
}


pub fn delete_event(client: &PgConnection, event_to_delete: i32) -> Result<()> {
    use self::schema::events::dsl::*;

    diesel::update(events.filter(event_id.eq(event_to_delete)))
        .set(deleted.eq(true))  // Soft delete.
        .execute(client)?;
    Ok(())
}


pub fn get_event(client: &PgConnection, event_to_get: i32) -> Result<Event> {
    use self::schema::events::dsl::*;

    let event: Option<Event> = events.filter(event_id.eq(event_to_get))
        .first::<Event>(client)
        .optional()?;
    match event {
        Some(e) => Ok(e),
        None => Err(ServerError::from(EventError::NoSuchEvent))
    }
}


pub struct Applicants {
    /// User id.
    pub uid: i32,
    /// Nickname.
    pub nick_name: String,
    /// Avatar.
    pub avatar: String,

    // TODO: implement real name field.
    // ..
}

// pub fn get_applicants(client: &PgConnection, event_to_query: i32) -> Result<Vec<Applicants>> {
//     // 返回所有 applicants. 后期 join 查询 OA Bindings
// }
// event::get_applicant
// event::pariticipate
// user::get_events
// event::sign