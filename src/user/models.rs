use super::schema::verifications;
use super::schema::oa_bindings;
use super::schema::persons;


pub const LOGIN_WECHAT: i32 = 0;
pub const LOGIN_USERNAME: i32 = 1;

pub const STATUS_NORMAL: i32 = 0;
pub const STATUS_DISABLED: i32 = 1;
pub const STATUS_NEW: i32 = 2;

pub const ROLE_NORMAL: i16 = 0;
pub const ROLE_ADMINISTRATOR: i16 = 1;


#[derive(Debug, Queryable, Insertable)]
#[table_name="verifications"]
pub struct Verification {
    pub id: i32,
    pub uid: i32,
    pub login_type: i32,
    pub account: String,
    pub credential: Option<String>,
}


#[derive(Debug, Queryable, Insertable)]
#[table_name="persons"]
pub struct Person {
    pub id: i32,
    pub uid: i32,
    pub sex: i32,
    pub real_name: Option<String>,
    pub nick_name: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar : Option<String>,
    pub profile: Option<String>,
    pub status: i32,
    pub country: Option<String>,
    pub province : Option<String>,
    pub city: Option<String>,
    pub role: i16,
}

#[derive(Insertable)]
#[table_name="persons"]
pub struct NewPerson {
    pub status: i32,
    pub role: i16,
}

#[derive(Queryable, Insertable)]
#[table_name="oa_bindings"]
pub struct OABinding {
    pub id: i32,
    pub uid: i32,
    pub student_id: Option<String>,
    pub oa_password: Option<String>,
    pub oa_certified: bool,
    pub class: Option<String>,
}


impl Verification {
    pub fn new(login_type: i32) -> Verification {
        Self {
            id: 0,
            uid: 0,
            login_type,
            account: "".to_string(),
            credential: None,
        }
    }
}

impl NewPerson
{
    pub fn new() -> NewPerson
    {
        NewPerson
        {
            status: STATUS_NEW,
            role: ROLE_NORMAL,
        }
    }
}