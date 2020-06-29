//! The server module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use std::fs::File;
use std::io::BufReader;

use actix_http::http::HeaderValue;
use actix_web::{web, App, HttpResponse, HttpServer};
use handlers::{attachment, freshman};
use rustls::internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::config::CONFIG;
use crate::error::Result;

mod handlers;
mod middlewares;
// User related interfaces.
mod auth;
mod jwt;

// TODO: Features
// - HTTP/2 supported
// - HTTPS
// - log to file / database
// The entrance of server is following.
#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let pool = PgPool::builder()
        .build(&CONFIG.db_string.as_ref())
        .await
        .expect("Could not create database pool");

    // load ssl keys
    let mut config = ServerConfig::new(NoClientAuth::new());
    // cert.pem is as same as fullchain.pem provided by let's encrypt.
    // and key.pem is privkey.pem
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    // Run actix-web server.
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middlewares::acl::Auth)
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hello world")))
            .service(freshman::get_basic_info)
            .service(freshman::update_account)
            .service(freshman::get_roommate)
            .service(freshman::get_classmate)
            .service(freshman::get_people_familiar)
    })
    .bind_rustls(&CONFIG.bind_addr.as_str(), config)?
    .run()
    .await
}

#[derive(Debug, Serialize)]
pub struct NormalResponse<T> {
    code: u16,
    pub data: T,
}

#[derive(Default, Serialize)]
struct EmptyReponse {
    pub code: u16,
}

impl<T> NormalResponse<T> {
    pub fn new(data: T) -> NormalResponse<T>
    where
        T: Serialize,
    {
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

/// User Jwt token carried in each request.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JwtToken {
    /// UID of current user.
    pub uid: i32,
    /// current user role.
    pub is_admin: bool,
}

fn get_auth_bearer_value(auth_string: &HeaderValue) -> Option<&str> {
    // https://docs.rs/actix-web/2.0.0/actix_web/http/header/struct.HeaderValue.html#method.to_str
    // Note: to_str().unwrap() will panic when value string contains non-visible chars.
    if let Ok(auth_string) = auth_string.to_str() {
        // Authorization: <Type> <Credentials>
        if auth_string.starts_with("Bearer ") {
            return Some(auth_string[7..].as_ref());
        }
    }
    None
}
