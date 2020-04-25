use chrono::NaiveDateTime;
use serde_json;
use tokio_postgres::Client;

use crate::error::{Result, ServerError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    publisher: i32,
    title: String,
    description: String,
    #[serde(rename = "startTime")]
    start_time: NaiveDateTime,
    #[serde(rename = "endTime")]
    end_time: NaiveDateTime,
    tag: String,
    #[serde(rename = "maxPeople")]
    max_people: i16,
    place: String,
    image: String,
    attachments: Vec<String>,
}

#[derive(Fail, Debug, ToPrimitive)]
pub enum EventError {
    #[fail(display = "重复创建活动")]
    DuplicatedEvent = 8,
    #[fail(display = "找不到这个活动")]
    NoSuchEvent = 9,
}


pub async fn create_event(client: &Client, event: &Event) -> Result<i32> {
    let statement = client.prepare(
        "INSERT INTO events (publisher_uid, title, description, start_time, end_time, tag, place, image, attachment)\
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING (event_id)").await?;
    let row = client.query_one(&statement, &[&event.publisher, &event.title, &event.description,
        &event.start_time, &event.end_time, &event.tag, &event.place, &event.image, &event.attachments]).await?;

    Ok(rows[0].try_get(0)?)
}

#[derive(Debug, Serialize)]
pub struct EventBase {
    event_id: i32,
    publisher: String,
    title: String,
    tag: String,
    place: String,
    #[serde(rename = "startTime")]
    start_time: NaiveDateTime,
    #[serde(rename = "endTime")]
    end_time: NaiveDateTime,
    image: String,
    attachments: Vec<String>,
}

pub async fn list_events(client: &Client, page_index: u32, page_size: u32) -> Result<Vec<EventBase>> {
    let statement = client.prepare(
        "SELECT e.event_id, p.nick_name as publisher, e.title, e.tag, e.place, e.start_time, e.end_time, e.image  \
         FROM events as e, persons as p \
         WHERE e.deleted is not true \
         OFFSET $1 LIMIT $2").await?;
    let rows = client.query(&statement, &[&((page_index - 1) * page_size), &page_size]).await?;
    let results = rows.iter().map(|row| {
        EventBase {
            event_id: row.get(0),
            publisher: row.get(1),
            title: row.get(2),
            tag: row.get(3),
            place: row.get(4),
            start_time: row.get(5),
            end_time: row.get(6),
            image: row.get(7),
            attachments: Vec::new(),
        }
    }).collect();

    Ok(results)
}


pub async fn delete_event(client: &Client) -> Result<()> {
    let statement = client.prepare("UPDATE events SET deleted = 1 WHERE event_id = $1").await?;
    // result is equal to the count of affected rows
    // TODO: check result to confirm the event is actually deleted.
    let result = client.execute(&statement, &[]).await?;

    Ok(())
}

pub async fn get_event(client: &Client, event_id: i32) -> Result<Event> {}
// event::delete
// event::get
// event::get_applicant
// event::pariticipate
// user::get_events
// event::sign