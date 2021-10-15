//! This module provides the ability to create, update and delete events, records and other about signs.
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::PageView;

/// Event that imported from OA.
const EVENT_TYPE_OA: i32 = 0;
/// Event which user pub in kite.
const EVENT_TYPE_INNER: i32 = 1;

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum EventError {
    #[error("重复创建活动")]
    DuplicatedEvent = 270,
    #[error("找不到这个活动")]
    NoSuchEvent = 271,
    #[error("重复申请")]
    DuplicatedApply = 272,
    #[error("重复签到")]
    AlreadySigned = 273,
    #[error("需要实名认证")]
    NeedIdentity = 274,
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
#[serde(rename_all = "camelCase")]
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
    pub start_time: NaiveDateTime,
    /// Event end time, in UTC+8.
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
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    pub source: i32,
    pub id: i32,
    pub publisher_uid: Option<i32>,
    pub publisher_name: Option<String>,
    pub title: String,
    pub start_time: NaiveDateTime,
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
    pub async fn create(&mut self, client: &PgPool) -> Result<()> {
        let event: Option<(i32,)> = sqlx::query_as(
            "INSERT INTO events.events
                (publisher_uid, title, description, start_time, end_time, tags, place, image)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING event_id",
        )
        .bind(&self.publisher_uid)
        .bind(&self.title)
        .bind(&self.description)
        .bind(&self.start_time)
        .bind(&self.end_time)
        .bind(&self.tags)
        .bind(&self.place)
        .bind(&self.image)
        .fetch_optional(client)
        .await?;
        if let Some((event_id_value,)) = event {
            self.id = event_id_value;
        }
        Ok(())
    }

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

#[derive(sqlx::FromRow, Debug)]
pub struct ScScore {
    pub category: i32,
    pub amount: f32,
}

pub async fn query_sc_score(db: &PgPool, query: &str) -> Result<Vec<ScScore>> {
    let score = sqlx::query_as(
        "SELECT category, sum(amount) as amount
        FROM events.sc_events as event , events.sc_detail as detail
        WHERE event.activity_id = detail.activity_id
          AND status = '通过'
          AND student_id = $1
        GROUP BY category;",
    )
    .bind(query)
    .fetch_all(db)
    .await?;

    Ok(score)
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScActivityList {
    pub activity_id: i32,
    pub title: String,
    pub start_time: DateTime<Local>,
    pub sign_end_time: DateTime<Local>,
    pub category: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScActivityDetail {
    pub activity_id: i32,
    pub category: i32,
    pub title: String,
    pub start_time: DateTime<Local>,
    pub sign_start_time: DateTime<Local>,
    pub sign_end_time: DateTime<Local>,
    pub place: Option<String>,
    pub duration: Option<String>,
    pub manager: Option<String>,
    pub contact: Option<String>,
    pub organizer: Option<String>,
    pub undertaker: Option<String>,
    pub description: String,
    pub image: Vec<String>,
}

pub async fn get_sc_activity_list(pool: &PgPool, page: &PageView) -> Result<Vec<ScActivityList>> {
    let result = sqlx::query_as(
        "SELECT activity_id, title, start_time, sign_end_time, category
        FROM events.sc_events
        ORDER BY start_time DESC
        LIMIT $1 OFFSET $2;",
    )
    .bind(page.count(20) as i32)
    .bind(page.offset(20) as i32)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn get_sc_activity_detail(pool: &PgPool, activity_id: i32) -> Result<ScActivityDetail> {
    let result = sqlx::query_as(
        "SELECT activity_id, title, start_time, sign_start_time, sign_end_time, place, duration,
         manager, contact, organizer, undertaker, description, image, category
        FROM events.sc_events
        WHERE activity_id = $1;",
    )
    .bind(activity_id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScActivityImage {
    pub id: Uuid,
    pub url: String,
}

pub async fn get_sc_image_url(pool: &PgPool, image_name: String) -> Result<ScActivityImage> {
    let result = sqlx::query_as(
        "SELECT id, url FROM attachments
        WHERE id = $1::uuid;",
    )
    .bind(image_name)
    .fetch_one(pool)
    .await?;

    Ok(result)
}
