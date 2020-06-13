use chrono::NaiveDateTime;
use diesel::prelude::*;
use futures::TryFutureExt;

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

pub fn list_events(
    client: &PgConnection,
    page_index: u32,
    page_size: u32,
) -> Result<Vec<EventSummary>> {
    use self::schema::events::dsl::*;

    let event_records: Vec<Event> = events
        .order(id.desc())
        .offset(((page_index - 1) * page_size) as i64)
        .limit(page_size as i64)
        .get_results::<Event>(client)?;
    // TODO: Add publish time for events.
    Ok(event_records
        .into_iter()
        .map(|event| event.summarize())
        .rev()
        .collect())
}

pub fn list_all_events(
    client: &PgConnection,
    page_index: u32,
    page_size: u32,
) -> Result<Vec<EventSummary>> {
    use self::schema::events::dsl::*;

    let event_records: Vec<Event> = events
        .order(id.desc())
        .filter(deleted.eq(false))
        .offset(((page_index - 1) * page_size) as i64)
        .limit(page_size as i64)
        .get_results::<Event>(client)?;
    // TODO: Add publish time for events.
    Ok(event_records
        .into_iter()
        .map(|event| event.summarize())
        .rev()
        .collect())
}

pub fn delete_event(client: &PgConnection, event_to_delete: i32) -> Result<()> {
    use self::schema::events::dsl::*;

    diesel::update(events.filter(event_id.eq(event_to_delete)))
        .set(deleted.eq(true)) // Soft delete.
        .execute(client)?;
    Ok(())
}

pub fn get_event(client: &PgConnection, event_to_get: i32) -> Result<Event> {
    use self::schema::events::dsl::*;

    let event: Option<Event> = events
        .filter(event_id.eq(event_to_get))
        .first::<Event>(client)
        .optional()?;
    match event {
        Some(e) => Ok(e),
        None => Err(ServerError::from(EventError::NoSuchEvent)),
    }
}

#[derive(Queryable)]
pub struct ApplicantResult {
    /// User id.
    pub uid: i32,
    /// Nickname.
    pub nick_name: String,
    /// Avatar.
    pub avatar: String,
    // TODO: implement real name field.
    // ..
    pub apply_time: NaiveDateTime,
    pub sign_time: Option<NaiveDateTime>,
    pub sign_type: Option<i32>,
}

pub fn get_applicants(
    client: &PgConnection,
    event_to_query: i32,
    page_index: i32,
    page_size: i32,
) -> Result<Vec<ApplicantResult>> {
    use self::schema::event_applicants::dsl as applicants_schema;
    use crate::schema::*;
    use crate::user::schema::persons::dsl as persons_schema;

    let join = applicants_schema::event_applicants.inner_join(persons_schema::persons);
    let applicants = join
        .select((
            applicants_schema::uid,
            persons_schema::nick_name,
            persons_schema::avatar,
            applicants_schema::apply_time,
            applicants_schema::sign_time,
            applicants_schema::sign_type,
        ))
        .load::<ApplicantResult>(client)?;

    Ok(applicants)
}

pub fn get_events_participated(
    client: &PgConnection,
    uid: i32,
    page_index: i32,
    page_size: i32,
) -> Result<Vec<EventSummary>> {
    use self::schema::event_applicants::dsl as applicants_schema;
    use self::schema::events::dsl as events_schema;

    let join = applicants_schema::event_applicants.inner_join(events_schema::events);
    let events = join
        .filter(applicants_schema::uid.eq(uid))
        .select((
            applicants_schema::event_id,
            events_schema::publisher_uid,
            events_schema::title,
            events_schema::tags,
            events_schema::place,
            events_schema::start_time,
            events_schema::end_time,
            events_schema::create_time,
            events_schema::image,
        ))
        .get_results::<EventSummary>(client)?;

    Ok(events)
}

#[derive(Deserialize, Insertable)]
#[table_name = "event_applicants"]
struct Applicant {
    pub id: i32,
    pub uid: i32,
    pub event_id: i32,
    pub apply_time: NaiveDateTime,
    pub sign_time: Option<NaiveDateTime>,
    pub sign_type: Option<i32>,
    pub finished: bool,
}

/// Register before the event starts.
pub fn register(client: &PgConnection, event_id: i32, uid: i32) -> Result<()> {
    use self::schema::event_applicants::dsl as applicants_schema;

    let new_participation = Applicant {
        id: 0,
        uid,
        event_id,
        apply_time: Utc::now().naive_local(),
        sign_time: None,
        sign_type: None,
        finished: false,
    };
    let _ = diesel::insert_into(applicants_schema::event_applicants)
        .values(&new_participation)
        .on_conflict((applicants_schema::event_id, applicants_schema::uid))
        .do_nothing()
        .execute(client)?;

    Ok(())
}

/// Participate and sign.
pub fn participate(client: &PgConnection, event_id: i32, uid: i32, sign_type: i32) -> Result<()> {
    use self::schema::event_applicants::dsl as applicants_schema;

    let current_time = Utc::now().naive_local();
    let new_participation = Applicant {
        id: 0,
        uid,
        event_id,
        apply_time: current_time,
        sign_time: Some(current_time),
        sign_type: Some(sign_type),
        finished: true,
    };

    // Note: There are three situations.
    // 1. No applied or signed record.
    // 2. Has applied but not signed.
    // 3. Signed (may be applied, doesn't matter).
    let _ = diesel::insert_into(applicants_schema::event_applicants)
        .values(&new_participation)
        .on_conflict((
            applicants_schema::event_id,
            applicants_schema::uid,
            applicants_schema::finished,
        ))
        .do_update()
        .set((
            applicants_schema::sign_time.eq(&current_time),
            applicants_schema::sign_type.eq(&sign_type),
            applicants_schema::finished.eq(&true),
        ))
        .execute(client)?;
    Ok(())
}
// user::get_events
