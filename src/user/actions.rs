// use diesel::prelude::*;
//
// use crate::error::{Result, ServerError, UserError};
//
// use super::schema;
// use super::*;
//
// pub fn get_default_avatar() -> &'static str {
//     "https://"
// }
//
// pub fn password_login(client: &PgConnection, username: &str, password: &str) -> Result<i32> {
//     use schema::authentication::dsl::*;
//
//     let result = authentication
//         .filter(login_type.eq(&_LOGIN_BY_PASSWORD))
//         .filter(account.eq(&username))
//         .filter(credential.eq(&password))
//         .select(uid)
//         .first::<i32>(client)
//         .optional()?;
//     match result {
//         Some(u) => Ok(u),
//         None => Err(ServerError::from(UserError::LoginFailed)),
//     }
// }
//
// pub fn wechat_login(client: &PgConnection, openid: &str) -> Result<i32> {
//     use schema::authentication::dsl::*;
//
//     let result = authentication
//         .filter(login_type.eq(&_LOGIN_BY_WECHAT))
//         .filter(account.eq(&openid))
//         .select(uid)
//         .first::<i32>(client)
//         .optional()?;
//     match result {
//         Some(u) => Ok(u),
//         None => Err(ServerError::from(UserError::LoginFailed)),
//     }
// }
//
// /// "static" method. User login by a given authentication structure.
// /// If login successfully, return uid as i32.
// pub fn login(client: &PgConnection, auth: &Authentication) -> Result<i32> {
//     return match auth.login_type {
//         _LOGIN_BY_PASSWORD => {
//             let credentials: &String = auth.credential.as_ref().unwrap();
//             password_login(client, &auth.account, credentials.as_str())
//         }
//         _LOGIN_BY_WECHAT => wechat_login(client, &auth.account),
//         _ => Err(ServerError::from(UserError::BadParameter)),
//     };
// }
//
// pub fn find_like(client: &PgConnection, key_nick_name: &str) -> Result<Vec<Person>> {
//     use schema::persons::dsl::*;
//
//     let users = persons
//         .filter(nick_name.like(key_nick_name))
//         .get_results::<Person>(client)?;
//
//     Ok(users)
// }
//
// pub fn find_one(client: &PgConnection, _uid: i32) -> Result<Person> {
//     use schema::persons::dsl::*;
//
//     let user = persons
//         .filter(uid.eq(_uid))
//         .first::<Person>(client)
//         .optional()?;
//
//     return match user {
//         Some(u) => Ok(u),
//         None => Err(ServerError::from(UserError::NoSuchUser)),
//     };
// }
//
// #[derive(Serialize)]
// pub struct UserExtra {
//     pub gender: Option<i32>,
//     #[serde(rename = "avatarUrl")]
//     pub avatar_url: Option<String>,
//     pub country: Option<String>,
//     pub province: Option<String>,
//     pub city: Option<String>,
//     pub language: Option<String>,
// }
//
// pub fn create(client: &PgConnection, new_nick_name: String, new_extra: &UserExtra) -> Result<i32> {
//     use schema::persons::dsl::*;
//
//     let new_extra = serde_json::to_value(new_extra)?; // Serialize the extra
//     let new_uid = diesel::insert_into(persons)
//         .values((&nick_name.eq(&new_nick_name), &extra.eq(new_extra)))
//         .returning(uid)
//         .get_result::<i32>(client)?;
//     Ok(new_uid)
// }
