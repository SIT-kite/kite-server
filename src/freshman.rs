use chrono::NaiveDateTime;
use diesel::pg::expression::array_comparison::any;
use diesel::prelude::*;
use diesel::update;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ServerError};

/// The content is generated automatically by diesel-cli
///
/// Command:
///  diesel print-schema --schema schema
mod freshman_schema {
    table! {
        freshman.students (student_id) {
            student_id -> Varchar,
            uid -> Nullable<Int4>,
            ticket -> Nullable<Varchar>,
            name -> Varchar,
            college -> Varchar,
            major -> Varchar,
            room -> Int4,
            building -> Varchar,
            bed -> Varchar,
            class -> Varchar,
            province -> Nullable<Varchar>,
            city -> Nullable<Varchar>,
            graduated_from -> Nullable<Varchar>,
            postcode -> Nullable<Int4>,
            private -> Bool,
            contact -> Nullable<Jsonb>,
            last_seen -> Nullable<Timestamp>,
            campus -> Varchar,
            counselor_name -> Varchar,
            counselor_tel -> Varchar,
            secret -> Varchar,
        }
    }
}

/// FreshmanEnv
/// Used to express campus, dormitory, counselor and other environment variables
/// for each new student.
/// Note: This structure is used to query only.
#[derive(Queryable, Serialize, Deserialize)]
pub struct FreshmanEnv {
    pub uid: Option<i32>,
    /// student id.
    #[serde(rename(serialize = "studentId"))]
    pub student_id: String,
    /// Freshman college
    pub college: String,
    /// Freshman major
    pub major: String,
    /// campus of Fengxian or Xuhui.
    pub campus: String,
    /// like "1号楼". For Xuhui has some buildings named like "南1号楼", we use a string.
    pub building: String,
    /// like "101"
    pub room: i32,
    /// like "101-1"
    pub bed: String,
    /// Counselor's name
    pub counselor_name: String,
    /// Counselor's telephone
    pub counselor_tel: String,
    /// Allow people in the same city access one's contact details.
    pub private: bool,
}

/// This structure is of one student, which can be used in
/// show their classmates, roommates and people they may recognize.
#[derive(Queryable, Deserialize)]
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
    // TODO: pub avatar: Option<String>,
    /// Contact detail like wechat, qq, telephone...
    pub contact: Option<serde_json::Value>,
}

/// Information about people you might know
#[derive(Queryable, Deserialize)]
pub struct PeopleFamiliar {
    /// Name of the people may recognize.
    pub name: String,
    /// City where the people in
    pub city: String,
    /// Avatar
    pub avatar: String,
    /// Contact details.
    pub contact: String,
}

#[derive(Debug, Fail, ToPrimitive)]
pub enum FreshmanError {
    #[fail(display = "无匹配的新生数据")]
    NoSuchAccount = 18,
    #[fail(display = "账户不匹配")]
    DismatchAccount = 19,
}

pub fn get_stduent_id_by_account(
    client: &PgConnection,
    account: &String,
    passwd: &String,
) -> Result<String> {
    use freshman_schema::students::dsl::*;

    let student_id_result: Option<String> = students
        .filter(
            secret.eq(&passwd).and(
                student_id
                    .eq(&account)
                    .or(name.eq(&account).or(ticket.eq(&account))),
            ),
        )
        .select((student_id))
        .get_result::<String>(client)
        .optional()?;

    student_id_result.ok_or(ServerError::new(FreshmanError::NoSuchAccount))
}

/// returning student id
pub fn bind_account(client: &PgConnection, _uid: i32, account: &String, passwd: &String) -> Result<String> {
    use freshman_schema::students::dsl::*;

    if let Ok(current_uid) = get_stduent_id_by_account(client, &account, &passwd) {
        return Err(ServerError::new(FreshmanError::DismatchAccount));
    }

    let student_id_result: String = diesel::update(students)
        .filter(
            secret.eq(&passwd).and(
                student_id
                    .eq(&account)
                    .or(name.eq(&account).or(ticket.eq(&account))),
            ).and(uid.is_not_null())
        )
        .set(uid.eq(_uid))
        .returning(student_id)
        .get_result::<String>(client)?;
    Ok(student_id_result)
}


pub fn get_env_by_uid(client: &PgConnection, _uid: i32) -> Result<FreshmanEnv> {
    use freshman_schema::students::dsl::*;

    let student_env: Option<FreshmanEnv> = students
        .filter(uid.eq(_uid))
        .select((
            uid,
            student_id,
            college,
            major,
            campus,
            building,
            room,
            bed,
            counselor_name,
            counselor_tel,
            private,
        ))
        .get_result::<FreshmanEnv>(client)
        .optional()?;

    match student_env {
        Some(e) => Ok(e),
        None => Err(ServerError::new(FreshmanError::NoSuchAccount)),
    }
}


pub fn get_env_firstly(
    client: &PgConnection,
    _uid: i32,
    account: String,
    passwd: String,
) -> Result<FreshmanEnv> {
    use freshman_schema::students::dsl::*;

    let student_env: Option<FreshmanEnv> = students
        .filter(
            // Match (Password && (Name || StudentId || Ticket))
            secret.eq(&passwd).and(
                student_id
                    .eq(&account)
                    .or(name.eq(&account).or(ticket.eq(&account))),
            ),
        )
        .select((
            uid,
            student_id,
            college,
            major,
            campus,
            building,
            room,
            bed,
            counselor_name,
            counselor_tel,
            private,
        ))
        .get_result::<FreshmanEnv>(client)
        .optional()?;

    match student_env {
        Some(e) => {
            if e.uid.unwrap() == _uid {
                Ok(e)
            } else {
                Err(ServerError::new(FreshmanError::DismatchAccount))
            }
        }
        None => Err(ServerError::new(FreshmanError::NoSuchAccount)),
    }
}

pub fn update_contact(
    client: &PgConnection,
    _uid: i32,
    _student_id: &String,
    new_contact: &serde_json::Value,
) -> Result<()> {
    use freshman_schema::students::dsl::*;

    let affected_rows = diesel::update(students)
        .set(contact.eq(new_contact))
        .filter(student_id.eq(_student_id).and(uid.eq(_uid)))
        .execute(client)?;

    if affected_rows == 0 {
        return Err(ServerError::new(FreshmanError::NoSuchAccount));
    }
    Ok(())
}

pub fn set_private(
    client: &PgConnection,
    _uid: i32,
    _student_id: &String,
    _private: bool,
) -> Result<()> {
    use freshman_schema::students::dsl::*;

    let affected_rows = diesel::update(students)
        .set(private.eq(_private))
        .filter(student_id.eq(_student_id).and(uid.eq(_uid)))
        .execute(client)?;

    if affected_rows == 0 {
        return Err(ServerError::new(FreshmanError::NoSuchAccount));
    }
    Ok(())
}

pub fn get_classmates(client: &PgConnection, _uid: i32, _student_id: &String) -> Result<Vec<NewMate>> {
    use freshman_schema::students::dsl::*;

    // SELECT * FROM students WHERE class = (SELECT class FROM students WHERE student_id = $1)"
    let classmates: Vec<NewMate> = students
        .filter(
            class.eq_any(
                students
                    .filter(student_id.eq(_student_id).and(uid.eq(_uid)))
                    .select(class)
                    .into_boxed(),
            ),
        )
        .select((
            college, major, name, province, building, room, bed, last_seen,
            /*avatar,*/ contact,
        ))
        .get_results::<NewMate>(client)?;
    Ok(classmates)
}

pub fn get_roommates(client: &PgConnection, _uid: i32, _student_id: &String) -> Result<Vec<NewMate>> {
    use freshman_schema::students::dsl::*;

    let self_env: Option<FreshmanEnv> = students
        .filter(uid.eq(_uid).and(student_id.eq(_student_id)))
        .select((
            uid,
            student_id,
            college,
            major,
            campus,
            building,
            room,
            bed,
            counselor_name,
            counselor_tel,
            private,
        ))
        .get_result::<FreshmanEnv>(client)
        .optional()?;

    match self_env {
        Some(current_env) => {
            let roommates: Vec<NewMate> = students
                .filter(room.eq(current_env.room))
                .select((
                    college, major, name, province, building, room, bed, last_seen,
                    /*avatar,*/ contact,
                ))
                .get_results::<NewMate>(client)?;
            Ok(roommates
                .into_iter()
                .filter(|x| x.building == current_env.building)
                .collect())
        }
        None => Err(ServerError::new(FreshmanError::NoSuchAccount)),
    }
}
