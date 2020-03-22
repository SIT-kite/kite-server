#![allow(non_snake_case)]

use serde::{Serialize, Deserialize};
use actix_web::{web, App, http, HttpRequest, HttpServer, Responder, HttpResponse};

async fn greet(req: HttpRequest) -> impl Responder {
    "Hello world!"
}

use super::user::models::*;
use actix_web::cookie::Cookie;

#[derive(Debug, Serialize, Deserialize)]
struct SessionStru {
    loginType: i32,
    wxCode: Option<String>,
    account: Option<String>,
    credential: Option<String>,
}


async fn login(form: web::Form<SessionStru>) -> HttpResponse {
    let  mut v = Verification::new(form.loginType);

    match form.loginType {
        LOGIN_USERNAME => {
            if None == form.account || None == form.credential {
                return HttpResponse::BadRequest().body("");
            }
            v.account = form.account.as_ref().unwrap().parse().unwrap();
            v.credential = form.credential.clone();
        },
        LOGIN_WECHAT => {
            if None == form.wxCode {
                return HttpResponse::BadRequest().body("");
            }
            v.account = form.wxCode.as_ref().unwrap().parse().unwrap();
        },
        _ => {
            return HttpResponse::BadRequest().body("");
        }
    }
    let uid = match v.login() {
        Ok(u) => u,
        Err(_) => { println!("error"); 0}
    };
    let cookie: Cookie = Cookie::build("token", format!("uid={}", uid)).finish();
    HttpResponse::Ok().cookie(cookie).body("")
}


#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            // .route("/{name}", web::get().to(greet))
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/session")
                            .route("", web::post().to(login))
                    )
//                    .service(
//                        web::scope("/user")
//                            .route("/{id}", web::)
//                    )
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}