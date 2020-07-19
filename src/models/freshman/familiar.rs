use crate::error::Result;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{postgres::PgQueryAs, PgPool};

/// This structure is of one student, which can be used in
/// show their classmates, roommates and people they may recognize.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct NewMate {
    /// Freshman college
    pub college: String,
    /// Freshman major
    pub major: String,
    /// Freshman name
    pub name: String,
    /// Province, with out postfix "省"
    pub province: Option<String>,
    /// like "1号楼". For Xuhui has some buildings named like "南1号楼", we use a string.
    pub building: String,
    /// like "101"
    pub room: i32,
    /// Bed number, like "202-1"
    pub bed: String,
    /// last time the user access freshman system.
    #[serde(rename(serialize = "lastSeen"))]
    pub last_seen: Option<NaiveDateTime>,
    /// Avatar of the user
    pub avatar: Option<String>,
    /// Contact detail like wechat, qq, telephone...
    pub contact: Option<String>,
}

/// Information about people you might know
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct PeopleFamiliar {
    /// Name of the people may recognize.
    pub name: String,
    /// College
    pub college: String,
    /// City where the people in
    pub city: Option<String>,
    /// last time the user access freshman system.
    #[serde(rename(serialize = "lastSeen"))]
    pub last_seen: Option<NaiveDateTime>,
    /// Avatar
    pub avatar: Option<String>,
    /// Contact details.
    pub contact: Option<String>,
}

pub async fn get_classmates_by_uid(client: &PgPool, uid: i32) -> Result<Vec<NewMate>> {
    let classmates: Vec<NewMate> = sqlx::query_as(
        "SELECT college, major, name, province, building, room, bed, last_seen, avatar, contact
            FROM freshman.students AS stu
            LEFT JOIN public.person AS person
            ON stu.uid = person.uid
            INNER JOIN (
                    SELECT class, student_id FROM freshman.students WHERE uid = $1 LIMIT 1
                ) self
            ON
                stu.class = self.class
                AND stu.student_id <> self.student_id
            ORDER BY stu.student_id ASC;",
    )
    .bind(uid)
    .fetch_all(client)
    .await?;

    Ok(classmates)
}

pub async fn get_roommates_by_uid(client: &PgPool, uid: i32) -> Result<Vec<NewMate>> {
    let roommates: Vec<NewMate> = sqlx::query_as(
        "SELECT college, major, name, province, stu.building, stu.room, bed, last_seen, avatar, contact
            FROM freshman.students AS stu
            LEFT JOIN public.person AS person
            ON stu.uid = person.uid
            INNER JOIN (
                    SELECT building, room, student_id FROM freshman.students WHERE uid = $1 LIMIT 1
                ) self
            ON
                stu.room = self.room
                AND stu.building = self.building
                AND stu.student_id <> self.student_id;",
    )
    .bind(uid)
    .fetch_all(client)
    .await?;

    Ok(roommates)
}

pub async fn get_people_familiar_by_uid(client: &PgPool, uid: i32) -> Result<Vec<PeopleFamiliar>> {
    let people_familiar: Vec<PeopleFamiliar> = sqlx::query_as(
        "SELECT DISTINCT name, college, stu.city, last_seen, avatar, contact
            FROM freshman.students AS stu
            LEFT JOIN public.person AS person
            ON stu.uid = person.uid
            INNER JOIN (
                    SELECT graduated_from, city, postcode, student_id FROM freshman.students WHERE uid = $1 LIMIT 1
                ) self
            ON
                ((stu.graduated_from = self.graduated_from)
                OR stu.city = self.city
                OR stu.postcode / 1000 = self.postcode / 1000)
                AND stu.visible = true
                AND stu.student_id <> self.student_id;",
    )
        .bind(uid)
        .fetch_all(client)
        .await?;

    Ok(people_familiar)
}
