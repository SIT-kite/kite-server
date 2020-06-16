//! The server module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use actix_http::http::HeaderValue;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use diesel::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;
use crate::error::Result;
use crate::jwt::*;

mod middlewares;
// User related interfaces.
mod acl;
mod attachment;
mod sign;
mod user;

// TODO: Features
// - HTTP/2 supported
// - HTTPS
// - log to file / database
// The entrance of server is following.
#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let manager = ConnectionManager::<PgConnection>::new(&CONFIG.db_string);
    let pool = r2d2::Pool::new(manager).expect("Could not create database pool");

    // Run actix-web server.
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middlewares::auth::Auth)
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Hello world")),
            )
            .service(user::login)
    })
        .bind(&CONFIG.bind_addr.as_str())?
        .run()
        .await
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

fn parse_auth_line(auth_string: &HeaderValue) -> Option<(&str, &str)> {
    // https://docs.rs/actix-web/2.0.0/actix_web/http/header/struct.HeaderValue.html#method.to_str
    // Note: to_str().unwrap() will panic when value string contains non-visible chars.
    if let Ok(auth_string) = auth_string.to_str() {
        // Authorization: <Type> <Credentials>
        let auth_array: Vec<&str> = auth_string.split(" ").collect();
        if auth_array.len() == 2 {
            return Some((auth_array[0].clone(), auth_array[1].clone()));
        }
    }
    None
}

pub fn get_uid_by_req(req: &HttpRequest) -> Option<i32> {
    if let Some(auth_string) = req.headers().get("Authorization") {
        let result = parse_auth_line(auth_string);
        if let Some((auth_type, auth_credential)) = result {
            // TODO: 对异常情况应该记录，做到心里有数
            if auth_type == "Bearer" {
                if let Some(claim_struct) = decode_jwt(auth_credential) {
                    return Some(claim_struct.uid);
                }
            }
        }
    }
    None
}
