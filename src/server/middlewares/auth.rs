use std::task::{Context, Poll};

use actix_http::http::{HeaderValue, Method};
use actix_service::{Service, Transform};
use actix_web::{Error, http, HttpResponse};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use futures::future::{Either, ok, Ready};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;
use crate::jwt::*;

pub struct Auth;

impl<S, B> Transform<S> for Auth
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service })
    }
}


pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for AuthMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // 检查请求的 path 和请求方法
        // 对可匿名访问的页面予以放行
        if check_anonymous_list(req.method(), req.path()) {
            return Either::Left(self.service.call(req));
        }
        let mut is_logged_in = false;
        // Get authentication header.
        if let Some(auth_string) = req.headers().get("Authorization") {
            let result = parse_auth_line(auth_string);
            // Unpack JWT to verify credential
            if let Some((auth_type, auth_credential)) = result {
                // TODO: 对异常情况应该记录，做到心里有数
                if auth_type == "Bearer" {
                    if validate_jwt(auth_credential) {
                        is_logged_in = true;
                    }
                }
            }
        }

        if is_logged_in {
            Either::Left(self.service.call(req))
        } else {
            Either::Right(ok(req.into_response(
                HttpResponse::Forbidden()
                    .body(r#"{"code": 503, "msg": "Login needed.", "data": {}}"#)
                    .into_body()
            )))
        }
    }
}

fn check_anonymous_list(method: &Method, path: &str) -> bool {
    match path {
        "/session" => true,
        "/user" => method == Method::POST,
        _ => {
            if path.ends_with("/authentication") && path.starts_with("/user/") {
                true
            } else {
                false
            }
        },
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

