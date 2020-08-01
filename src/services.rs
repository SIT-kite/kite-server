//! The services module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use std::fs::File;
use std::io::{BufReader, Read};

use crate::config::CONFIG;
use crate::services::handlers::{attachment, checking, edu, event, freshman, motto, user};
use crate::services::middlewares::reject::Reject;
use actix_files::Files;
use actix_http::http::HeaderValue;
use actix_web::{web, App, HttpResponse, HttpServer};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

mod auth;
mod handlers;
mod middlewares;
pub(crate) mod response;

#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    // load ssl keys
    let mut config = ServerConfig::new(NoClientAuth::new());
    // cert.pem is as same as fullchain.pem provided by let's encrypt.
    // and key.pem is privkey.pem
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    // Create database pool.
    let pool = PgPool::builder()
        .build(&CONFIG.db_string.as_ref())
        .await
        .expect("Could not create database pool");

    // Logger
    set_logger("kite.log");
    let log_string = "%a - - [%t] \"%r\" %s %b %D \"%{User-Agent}i\"";

    // Load white list
    let mut file = std::fs::File::open("ip-whitelist.txt").unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    drop(file);

    // Run actix-web services.
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(middlewares::acl::Auth)
            .wrap(actix_web::middleware::Logger::new(log_string))
            .wrap(Reject::new(buffer.as_str()))
            .service(
                web::scope("/api/v1")
                    .route("/", web::get().to(|| HttpResponse::Ok().body("Hello world")))
                    .service(user::login)
                    .service(user::bind_authentication)
                    .service(user::list_users)
                    .service(user::create_user)
                    .service(user::get_user_detail)
                    .service(user::get_user_identity)
                    .service(user::set_user_identity)
                    .service(freshman::get_basic_info)
                    .service(freshman::update_account)
                    .service(freshman::get_roommate)
                    .service(freshman::get_classmate)
                    .service(freshman::get_people_familiar)
                    .service(attachment::index)
                    .service(attachment::upload_file)
                    .service(attachment::get_attachment_list)
                    .service(motto::get_one_motto)
                    .service(event::list_events)
                    .service(checking::list_student_status)
                    .service(checking::query_detail)
                    .service(checking::add_approval)
                    .service(checking::delete_approval)
                    .service(checking::list_admin)
                    .service(checking::delete_admin)
                    .service(checking::change_approval)
                    .service(edu::get_planned_course)
                    .service(edu::query_major)
                    .service(edu::list_course_classes)
                    .service(edu::query_course),
            )
            .service(Files::new("/static", &CONFIG.attachment_dir))
            .service(
                Files::new("/console/", "console/")
                    .index_file("index.html")
                    .use_etag(true)
                    .use_last_modified(true),
            )
    })
    .bind_rustls(&CONFIG.bind_addr.as_str(), config)?
    .run()
    .await
}

fn set_logger(path: &str) {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Info)
        // .chain(std::io::stdout())
        .chain(fern::log_file(path).expect("Could not open log file."))
        .apply()
        .expect("Failed to set logger.");
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
