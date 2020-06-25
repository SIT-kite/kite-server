// use diesel::{Insertable, Queryable};
// ///! This module provides the ability to create, update and delete users including authentication tokens.
// use serde::Serialize;
//
// // Reuse interfaces in actions mod.
// pub use actions::*;
// use schema::{authentication, persons};
//
// /// Interfaces.
// mod actions;
// /// Database schema.
// pub(crate) mod schema;
// /// Wechat ability
// pub(crate) mod wechat;
//
// /* Constants at the edge between self and database. */
//
// /// Login Type.
// #[allow(non_snake_case)]
// const _LOGIN_BY_WECHAT: i32 = 0;
// #[allow(non_snake_case)]
// const _LOGIN_BY_PASSWORD: i32 = 1;
//
// /* Models */
//
// /// Authentication structure, similar to table "authentication" in database.
// /// Record everybody's login credentials.
// #[derive(Default, Debug, Insertable, Queryable)]
// #[table_name = "authentication"]
// pub struct Authentication {
//     /// Record number, an increment ID
//     id: i32,
//     /// Target user.
//     pub uid: i32,
//     /// login type.
//     pub login_type: i32,
//     /// Username or wechat token (id).
//     pub account: String,
//     /// Password if uses username.
//     pub credential: Option<String>,
// }
//
// /// Base information of each account.
// #[derive(Default, Debug, Insertable, Queryable)]
// #[table_name = "persons"]
// pub struct Person {
//     /// Target user, key.
//     pub uid: i32,
//     /// Nickname. For users uses wechat to register, use wehcat name by default.
//     pub nick_name: String,
//     /// User avatar url.
//     pub avatar: String,
//     /// Is disabled. False by default.
//     pub is_disabled: bool,
//     /// Is administrator. False by default.
//     pub is_admin: bool,
//     /// User extra attributes.
//     pub extra: Option<serde_json::Value>,
//     // TODO: create_time
// }
