use diesel::{Insertable, Queryable};
///! This module provides the ability to create, update and delete users including authentication tokens.
use serde::Serialize;

// Reuse interfaces in actions mod.
pub use actions::*;
use schema::{authentication, persons};

/// Interfaces.
mod actions;
/// Database schema.
mod schema;
/// Wechat ability
pub(crate) mod wechat;

/* Constants at the edge between self and database. */

/// Login Type.
const _LOGIN_BY_WECHAT: i32 = 0;
const _LOGIN_BY_PASSWORD: i32 = 1;

/* Models */

/// Authentication structure, similar to table "authentication" in database.
/// Record everybody's login credentials.
#[derive(Default, Debug, Insertable, Queryable)]
#[table_name = "authentication"]
pub struct Authentication {
    /// Record number, an increment ID
    id: i32,
    /// Target user.
    pub uid: i32,
    /// login type.
    pub login_type: i32,
    /// Username or wechat token (id).
    pub account: String,
    /// Password if uses username.
    pub credential: Option<String>,
}

/// Base information of each account.
#[derive(Default, Debug, Insertable, Queryable)]
#[table_name = "persons"]
pub struct Person {
    /// Record number, an increment ID
    id: i32,
    /// Target user, key.
    pub uid: i32,
    /// Nickname. For users uses wechat to register, use wehcat name by default.
    pub nick_name: String,
    /// Is disabled. False by default.
    pub is_disabled: bool,
    /// Is administrator. False by default.
    pub is_admin: bool,
    /// User extra attributes.
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct NormalResponse<T> {
    code: u16,
    pub data: T,
}

#[derive(Serialize)]
struct EmptyReponse;

impl<T> NormalResponse<T> {
    pub fn new(data: T) -> NormalResponse<T> {
        NormalResponse { code: 0, data }
    }
}

impl<T> ToString for NormalResponse<T>
where
    T: Serialize,
{
    fn to_string(&self) -> String {
        if let Ok(body_json) = serde_json::to_string(&self) {
            return body_json;
        }
        r"{code: 1}".to_string()
    }
}
