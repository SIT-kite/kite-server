//! The services module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use crate::bridge::AgentManager;
use crate::config::CONFIG;
use actix_http::http::HeaderValue;
use actix_web::{web, App, HttpResponse, HttpServer};
use middlewares::reject::Reject;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::io::Read;

mod auth;
mod handlers;
mod middlewares;
mod response;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    host: AgentManager,
}

pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&CONFIG.server.db.as_ref())
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

    // Websocket server.
    let ws_host = AgentManager::new();

    let app_state = AppState {
        pool: pool,
        host: ws_host.clone(),
    };

    tokio::spawn(async move {
        ws_host.agent_main().await.unwrap_or_else(|e| {
            panic!("Failed to run websocket host: {}", e);
        });
    });

    // Run actix-web services.
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(middlewares::acl::Auth)
            .wrap(actix_web::middleware::Logger::new(log_string))
            .wrap(Reject::new(&buffer))
            .data(app_state.clone())
            .configure(routes)
    })
    .bind(&CONFIG.server.bind.as_str())?
    .run()
    .await
}

fn routes(app: &mut web::ServiceConfig) {
    use handlers::{attachment, edu, event, freshman, motto, notice, pay, status, user};

    app.service(
        // API scope: version 1
        web::scope("/api/v1")
            // API index greet :D
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hello world")))
            // User routes
            .service(user::login)
            .service(user::bind_authentication)
            .service(user::list_users)
            .service(user::create_user)
            .service(user::get_user_detail)
            .service(user::update_user_detail)
            .service(user::get_user_identity)
            .service(user::set_user_identity)
            // Freshman routes
            .service(freshman::get_basic_info)
            .service(freshman::update_account)
            .service(freshman::get_roommate)
            .service(freshman::get_classmate)
            .service(freshman::get_people_familiar)
            .service(freshman::get_analysis_data)
            .service(freshman::post_analysis_log)
            // Attachment routes
            .service(attachment::query_attachment)
            .service(attachment::upload_file)
            .service(attachment::list_attachments)
            // Motto routes
            .service(motto::get_one_motto)
            // Event and activity routes
            .service(event::list_events)
            // Edu management and course-related routes
            .service(edu::get_planned_course)
            .service(edu::query_major)
            .service(edu::list_course_classes)
            .service(edu::query_course)
            // System status routes
            .service(status::get_timestamp)
            .service(status::get_system_status)
            .service(status::get_agent_list)
            // Pay and room balance
            .service(pay::query_room_balance)
            .service(pay::query_room_bills)
            // Get Notices
            .service(notice::get_notices),
    );
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

use serde::{Deserialize, Serialize};

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
