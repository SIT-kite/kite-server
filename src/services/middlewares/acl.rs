use std::result::Result;

use actix_service::{Service, Transform};
use actix_utils::future::{self, ok, Ready};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
    Error,
};
use futures_util::future::Either;

use crate::error::ApiError;
use crate::jwt::*;
use crate::models::CommonError;
use crate::services::{get_auth_bearer_value, JwtToken};

pub struct Auth;

impl<S> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(AuthMiddleware { service })
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
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
        Either::Right(ok(req.error_response(ApiError::new(CommonError::LoginNeeded))))
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
        "/edu/schedule" => true,
        "/edu/calendar" => true,
        _ => {
            method == Method::GET
                && (path.starts_with("/static/")
                    || path.starts_with("/console/")
                    || path.starts_with("/api/v1/status/")
                    || path.starts_with("/api/v1/search/"))
        }
    }
}
