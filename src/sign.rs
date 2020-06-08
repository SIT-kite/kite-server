//! This module provides the ability to create, update and delete events, records and other about signs.
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use schema::events;

pub mod event;
mod schema;

/* Models */

/// A event which corresponds to one activity.
#[derive(Debug, Queryable, Insertable, Serialize, Deserialize)]
pub struct Event {
    /// ID in inner database.
    id: i32,
    /// Event id.
    pub event_id: i32,
    /// uid of event publisher.
    pub publisher_uid: i32,
    /// The name of the event.
    pub title: String,
    /// Full description of the event.
    pub description: String,
    /// Event start time, in UTC+8.
    #[serde(rename = "startTime")]
    pub start_time: NaiveDateTime,
    /// Event end time, in UTC+8.
    #[serde(rename = "endTime")]
    pub end_time: Option<NaiveDateTime>,
    #[serde(rename = "createTime")]
    pub create_time: NaiveDateTime,
    /// Tags, used to mark club name, activity classification.
    pub tags: Option<Vec<String>>,
    pub deleted: bool,
    /// Max count of members.
    #[serde(rename = "maxPeople")]
    pub max_people: Option<i16>,
    /// Name of exact location.
    pub place: String,
    /// Preview thumbnail in the event list.
    pub image: String,
    /// Big picture of event details.
    pub attachments: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSummary {
    pub event_id: i32,
    /// uid of event publisher.
    pub publisher_uid: i32,
    /// The name of the event.
    pub title: String,
    /// Tags, used to mark club name, activity classification.
    pub tags: Option<Vec<String>>,
    /// Name of exact location.
    pub place: String,
    /// Event start time, in UTC+8.
    #[serde(rename = "startTime")]
    pub start_time: NaiveDateTime,
    /// Event end time, in UTC+8.
    #[serde(rename = "endTime")]
    pub end_time: Option<NaiveDateTime>,
    #[serde(rename = "createTime")]
    pub create_time: NaiveDateTime,
    /// Preview thumbnail in the event list.
    pub image: String,
}

impl Event {
    pub fn default() -> Event {
        Event {
            id: 0,
            event_id: 0,
            publisher_uid: 0,
            title: "".to_string(),
            description: "".to_string(),
            start_time: Utc::now().naive_local(),
            end_time: None,
            create_time: Utc::now().naive_local(),
            tags: None,
            deleted: false,
            max_people: None,
            place: "".to_string(),
            image: "".to_string(),
            attachments: None,
        }
    }
}

trait Summarize<T> {
    fn summarize(&self) -> T;
}

impl Summarize<EventSummary> for Event {
    fn summarize(&self) -> EventSummary {
        EventSummary {
            event_id: self.event_id,
            publisher_uid: self.publisher_uid,
            title: self.title.clone(),
            tags: self.tags.clone(),
            place: self.place.clone(),
            start_time: self.start_time,
            end_time: self.end_time,
            create_time: self.create_time,
            image: self.image.clone(),
        }
    }
}

impl Into<Event> for EventSummary {
    fn into(self) -> Event {
        Event {
            event_id: self.event_id,
            publisher_uid: self.publisher_uid,
            title: self.title.clone(),
            tags: self.tags.clone(),
            place: self.place.clone(),
            start_time: self.start_time,
            end_time: self.end_time,
            image: self.image.clone(),
            create_time: self.create_time,
            ..Event::default()
        }
    }
}
