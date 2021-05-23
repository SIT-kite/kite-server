//! This module provides the ability to create, update and delete events, records and other about signs.
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::Result;

/// Event that imported from OA.
const EVENT_TYPE_OA: i32 = 0;
/// Event which user pub in kite.
const EVENT_TYPE_INNER: i32 = 1;

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum EventError {
    #[error("重复创建活动")]
    DuplicatedEvent = 8,
    #[error("找不到这个活动")]
    NoSuchEvent = 9,
    #[error("重复申请")]
    DuplicatedApply = 14,
    #[error("重复签到")]
    AlreadySigned = 17,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ApplicantRecord {
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

#[derive(Debug, sqlx::FromRow)]
struct Applicant {
    pub id: i32,
    pub uid: i32,
    pub event_id: i32,
    pub apply_time: NaiveDateTime,
    pub sign_time: Option<NaiveDateTime>,
    pub sign_type: Option<i32>,
    pub finished: bool,
}

/// A event which corresponds to one activity.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    /// Event type.
    pub source: i32,
    /// Event id.
    pub id: i32,
    /// uid of event publisher.
    pub publisher_uid: Option<i32>,
    /// manager or nick_name of publisher
    pub publisher_name: Option<String>,
    /// The name of the event.
    pub title: String,
    /// Event start time, in UTC+8.
    #[serde(rename = "startTime")]
    pub start_time: NaiveDateTime,
    /// Event end time, in UTC+8.
    #[serde(rename = "endTime")]
    pub end_time: Option<NaiveDateTime>,
    /// Tags, used to mark club name, activity classification.
    pub tags: Option<Vec<String>>,
    /// Name of exact location.
    pub place: String,
    /// Title image.
    pub image: Option<String>,
    /// Description
    pub description: String,
}

/// Summary of event. See strcut Event for details.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventSummary {
    pub source: i32,
    pub id: i32,
    pub publisher_uid: Option<i32>,
    pub publisher_name: Option<String>,
    pub title: String,
    #[serde(rename = "startTime")]
    pub start_time: NaiveDateTime,
    #[serde(rename = "endTime")]
    pub end_time: Option<NaiveDateTime>,
    pub tags: Option<Vec<String>>,
    pub place: String,
    pub image: Option<String>,
}

trait Summarize<T> {
    fn summarize(self) -> T;
}

impl Summarize<EventSummary> for Event {
    fn summarize(self) -> EventSummary {
        EventSummary {
            id: self.id,
            source: self.source,
            publisher_uid: self.publisher_uid,
            publisher_name: self.publisher_name,
            title: self.title,
            tags: self.tags,
            place: self.place,
            start_time: self.start_time,
            end_time: self.end_time,
            image: self.image,
        }
    }
}

impl Event {
    /// Create a new event struct (object).
    pub fn new() -> Self {
        Event { ..Event::default() }
    }
    /// Save event to database
    pub async fn create(_client: &PgPool) {}

    pub async fn list(client: &PgPool, page_index: u32, count: u32) -> Result<Vec<EventSummary>> {
        let count = if count > 50 { 50 } else { count };

        let events: Vec<EventSummary> = sqlx::query_as(
            "SELECT source, id, publisher_uid, publisher_name, title, start_time, end_time, tags, place, image
                FROM events.all_events
                OFFSET $1 LIMIT $2;")
            .bind(((page_index - 1) * count) as i32)
            .bind(count as i32)
            .fetch_all(client)
            .await?;
        Ok(events)
    }

    pub async fn get_event_detail(_source: i32) {}
}

impl Event {
    pub fn default() -> Event {
        Event {
            source: 0,
            id: 0,
            publisher_uid: None,
            publisher_name: None,
            title: "".to_string(),
            description: "".to_string(),
            start_time: Utc::now().naive_local(),
            end_time: None,
            tags: None,
            place: "".to_string(),
            image: None,
        }
    }
}
