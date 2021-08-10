use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use futures::future::{ok, Either, Ready};

use crate::error::ApiError;
use crate::ipset;
use crate::models::CommonError;

pub struct Reject {
    white_list: ipset::IpSet,
}

impl Reject {
    pub fn new(text: &str) -> Self {
        let mut ip_set = ipset::IpSet::new();

        ip_set.load(text);
        Self { white_list: ip_set }
    }
}

impl<S> Transform<S, ServiceRequest> for Reject
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = RejectMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RejectMiddleware {
            service,
            white_list: self.white_list.clone(),
        })
    }
}

pub struct RejectMiddleware<S> {
    service: S,
    white_list: ipset::IpSet,
}

impl<S> Service<ServiceRequest> for RejectMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // X-Forwarded-For field is set by nginx, so we should trust it.
        let origin_addr = req.headers().get("X-Forwarded-For");

        if let Some(peer_addr) = origin_addr.and_then(|peer| peer.to_str().ok()) {
            if let Ok(ipv4_addr) = Ipv4Addr::from_str(peer_addr) {
                let should_allow = self
                    .white_list
                    .contain(ipset::convert_ipv4_addr_to_u32(&ipv4_addr.octets()));

                if should_allow {
                    return Either::Left(self.service.call(req));
                }
            } else if Ipv6Addr::from_str(peer_addr).is_ok() {
                // Allow ipv6 request unconditionally.
                return Either::Left(self.service.call(req));
            }
        }
        Either::Right(ok(
            req.error_response(ApiError::new(CommonError::AddrNotSupported))
        ))
    }
}
