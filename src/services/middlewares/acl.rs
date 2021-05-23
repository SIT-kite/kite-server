use std::result::Result;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
    Error, HttpResponse,
};
use futures::future::{ok, Either, Ready};

use crate::error::ApiError;
use crate::jwt::*;
use crate::models::CommonError;
use crate::services::{get_auth_bearer_value, JwtToken};

pub struct Auth;

impl<S, B> Transform<S> for Auth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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

        // For logined users, they can access all of the resources, and then each module will check
        // whether they can do or not.
        if let Some(auth_string) = req.headers().get("Authorization") {
            // If authentication type is "Bearer"
            if let Some(jwt_string) = get_auth_bearer_value(auth_string) {
                // Unpack JWT to verify credential
                if validate_jwt::<JwtToken>(jwt_string) {
                    return Either::Left(self.service.call(req));
                }
            }
        }
        Either::Right(ok(req.into_response(
            HttpResponse::Ok()
                .json(ApiError::new(CommonError::LoginNeeded))
                .into_body(),
        )))
    }
}

fn check_anonymous_list(method: &Method, path: &str) -> bool {
    // TODO: Use regex expression.
    match path {
        "/" => true,
        "/api/v1/" => true,
        "/api/v1/session" => method == Method::POST,
        "/api/v1/user" => method == Method::POST,
        "/api/v1/event" => method == Method::GET,
        "/api/v1/motto" => method == Method::GET,
        "/agent/" => true,
        "/api/v1/notice" => true,
        _ => {
            path.starts_with("/static/")
                || path.starts_with("/console/")
                || path.starts_with("/api/v1/status/")
        }
    }
}
