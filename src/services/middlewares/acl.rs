use std::collections::HashMap;
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

const URL_WHITE_LIST: [(&str, Method); 8] = [
    ("/api/v1/", Method::GET),
    ("/api/v1/session", Method::POST),
    ("/api/v1/user", Method::POST),
    ("/api/v1/event", Method::GET),
    ("/api/v1/motto", Method::GET),
    ("/api/v1/notice", Method::GET),
    ("/api/v1/edu/schedule", Method::GET),
    ("/api/v1/edu/calendar", Method::GET),
];

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
        let mut no_checking_urls = HashMap::new();

        for (path, method) in URL_WHITE_LIST.iter() {
            no_checking_urls.insert(path.clone(), method.clone());
        }
        future::ok(AuthMiddleware {
            service,
            no_checking_urls,
        })
    }
}

pub struct AuthMiddleware<S> {
    service: S,
    no_checking_urls: HashMap<&'static str, Method>,
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
        let url = req.path();
        if let Some(method) = self.no_checking_urls.get(url) {
            if method == req.method() {
                return Either::Left(self.service.call(req));
            }
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
