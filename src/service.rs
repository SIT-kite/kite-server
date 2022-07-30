//! The services module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use poem::http::Method;
use poem::middleware::{AddData, Cors};
use poem::{get, listener::TcpListener, patch, post, put, EndpointExt, Route, Server};
use reqwest::redirect::Policy;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;

use jwt::JwtToken;

use crate::config::CONFIG;
use crate::middleware::logger::Logger;

mod badge;
mod classroom;
mod contact;
mod electricity;
mod freshman;
mod game;
mod jwt;
mod library;
mod notice;
mod report;
mod user;
mod weather;

fn create_route() -> Route {
    use classroom::*;
    use contact::*;
    use electricity::*;
    use notice::*;
    use report::*;
    use user::*;
    use weather::*;

    let route = Route::new()
        .at("/report/exception", post(post_exception))
        .at("/report/event", post(post_user_event))
        .at("/session", post(login))
        .at("/notice", get(get_notice_list))
        .at("/contact", get(query_all_contacts))
        .at("/weather/:campus", get(get_weather))
        .at("/classroom/available", get(query_available_classrooms))
        .nest(
            "/electricity",
            Route::new()
                .at("/room/:room", get(query_room_balance))
                .at("/room/:room/rank", get(query_room_consumption_rank))
                .at("/room/:room/bill/days", get(query_room_bills_by_day))
                .at("/room/:room/bill/hours", get(query_room_bills_by_hour)),
        )
        .nest(
            "/badge",
            Route::new()
                .at("/card/", get(badge::get_my_cards))
                .at("/result", get(badge::get_event_result))
                .at("/share", post(badge::append_share_log)),
        )
        .nest(
            "/game",
            Route::new()
                .at("/ranking/:game", get(game::get_ranking))
                .at("/record", post(game::post_record)),
        )
        .nest(
            "/library",
            Route::new()
                .at("/notice", get(library::get_notice))
                .at("/status/:date/", get(library::get_status))
                .at("/application/", get(library::get_application_list))
                .at("/publicKey", get(library::get_public_key))
                .at("/application/:apply_id/code", get(library::get_code))
                .at("/application", post(library::apply))
                .at(
                    "/application/:apply_id",
                    patch(library::update_application_status).delete(library::cancel),
                )
                .at("/current", get(library::get_current_period)),
        )
        .nest(
            "/freshman",
            Route::new()
                .at("/:account", get(freshman::get_basic_info))
                .at("/:account", put(freshman::update_account))
                .at("/:account/roommate", get(freshman::get_roommate))
                .at("/:account/familiar", get(freshman::get_people_familiar))
                .at("/:account/classmate", get(freshman::get_classmate))
                .at("/:account/analysis", get(freshman::get_analysis_data))
                .at("/:account/analysis/log", post(freshman::post_analysis_log)),
        );
    Route::new().nest("/v2", route)
}

pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .connect(CONFIG.get().unwrap().db.as_str())
        .await
        .expect("Could not create database pool");

    // Global http client.
    let mut client_builder = reqwest::Client::builder().redirect(Policy::none());
    if let Some(proxy) = &CONFIG.get().unwrap().http_proxy {
        client_builder = client_builder
            .proxy(reqwest::Proxy::http(proxy).unwrap())
            .proxy(reqwest::Proxy::https(proxy).unwrap())
            .danger_accept_invalid_certs(true);
    }
    let client = client_builder.build().unwrap();

    // Start weather update daemon
    use crate::model::weather;
    tokio::spawn(weather::weather_daemon(pool.clone()));

    // Run poem services
    let route = create_route();
    let app = route
        .with(AddData::new(pool))
        .with(AddData::new(client))
        .with(Logger)
        .with(
            Cors::new()
                .allow_origins(["https://cdn.kite.sunnysab.cn", "https://kite.sunnysab.cn"])
                .allow_methods([Method::POST, Method::GET, Method::OPTIONS]),
        );
    Server::new(TcpListener::bind(CONFIG.get().unwrap().bind.as_str()))
        .name("kite-server-v2")
        .run(app)
        .await
}
