use actix_http::{Error, Payload, PayloadStream};
use actix_web::{FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use serde::Deserialize;

use crate::server::JwtToken;

use super::get_auth_bearer_value;
use super::jwt::{decode_jwt, validate_jwt};
use actix_web::error::ErrorUnauthorized;

impl FromRequest for JwtToken {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        // Get authentication header.
        if let Some(auth_string) = req.headers().get("Authorization") {
            // If authentication type is "Bearer"
            if let Some(jwt_string) = get_auth_bearer_value(auth_string) {
                // Unpack JWT to verify credential
                if let Some(token) = decode_jwt::<JwtToken>(jwt_string) {
                    return ok(token);
                }
            }
        }
        err(ErrorUnauthorized("Unauthorized"))
    }
}
