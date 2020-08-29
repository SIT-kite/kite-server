//! This is the freshman module, which is a part of sit-kite project.
//! Freshman module, as a tool, allows freshmen query their dormitory, counselor
//! and classmates.
//! In the design of this module, we use word "account" to express student id,
//! name or admission ticket number, when the word "secret" used as "password".
//! Usually, secret is the six right characters of their id card number.

mod familiar;
mod myself;

use chrono::NaiveDateTime;
use serde::Serialize;

pub use familiar::*;
pub use myself::*;

#[derive(Debug, Fail, ToPrimitive)]
pub enum FreshmanError {
    #[fail(display = "无匹配的新生数据")]
    NoSuchAccount = 120,
    #[fail(display = "账户不匹配")]
    DismatchAccount = 121,
    #[fail(display = "已绑定")]
    BoundAlready = 122,
    #[fail(display = "需要凭据")]
    SecretNeeded = 123,
}

/// FreshmanBasic
///
/// Used to express campus, dormitory, counselor and other environment variables
/// for each new student.
/// Note: This structure is used to query only.
#[derive(sqlx::FromRow, Serialize)]
pub struct FreshmanBasic {
    pub name: String,
    pub uid: Option<i32>,
    /// student id.
    #[serde(rename(serialize = "studentId"))]
    pub student_id: String,
    /// Secret. Usually it's the right 6 bits of id number.
    #[serde(skip_serializing)]
    pub secret: String,
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
    #[serde(rename(serialize = "counselorName"))]
    pub counselor_name: String,
    /// Counselor's telephone
    #[serde(rename(serialize = "counselorTel"))]
    pub counselor_tel: String,
    /// Allow people in the same city access one's contact details.
    pub visible: bool,
}

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
    /// Gender. 'M' for boys and 'F' for girls.
    pub gender: String,
    /// last time the user access freshman system.
    #[serde(rename(serialize = "lastSeen"))]
    pub last_seen: Option<NaiveDateTime>,
    /// Avatar of the user
    pub avatar: Option<String>,
    /// Contact detail like wechat, qq, telephone...
    pub contact: Option<String>,
}

/// Information about people you might know
#[derive(sqlx::FromRow, Serialize)]
pub struct PeopleFamiliar {
    /// Name of the people may recognize.
    pub name: String,
    /// College
    pub college: String,
    /// City where the people in
    pub city: Option<String>,
    /// Gender. 'M' for boys and 'F' for girls.
    pub gender: String,
    /// last time the user access freshman system.
    #[serde(rename(serialize = "lastSeen"))]
    pub last_seen: Option<NaiveDateTime>,
    /// Avatar
    pub avatar: Option<String>,
    /// Contact details.
    pub contact: Option<String>,
}

#[derive(Serialize)]
pub struct GenderAnalysis {
    pub total: i64,
    pub boys: i64,
    pub girls: i64,
}

#[derive(Serialize)]
pub struct FreshmanAnalysis {
    #[serde(rename = "sameName")]
    pub same_name: i64,
    #[serde(rename = "sameCity")]
    pub same_city: i64,
    #[serde(rename = "sameHighSchool")]
    pub same_high_school: i64,
    #[serde(rename = "collegeCount")]
    pub college_count: i64,
    pub major: GenderAnalysis,
}

trait MapDefaultAvatar {
    fn map_default_avatar(self) -> Self;
}

use super::user::get_default_avatar;

macro_rules! impl_default_avatar {
    ($structure: ident) => {
        impl MapDefaultAvatar for Vec<$structure> {
            fn map_default_avatar(self) -> Vec<$structure> {
                self.into_iter()
                    .map(|mut x| {
                        x.avatar = Some(x.avatar.unwrap_or(get_default_avatar().to_string()));
                        x
                    })
                    .collect()
            }
        }
    };
}

impl_default_avatar!(NewMate);
impl_default_avatar!(PeopleFamiliar);
