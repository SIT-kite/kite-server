use std::net::Ipv4Addr;
use std::str::FromStr;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
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

impl<S, B> Transform<S> for Reject
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
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

impl<S, B> Service for RejectMiddleware<S>
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
        // X-Forwarded-For field is set by nginx, so we should trust it.
        let origin_addr = req.headers().get("X-Forwarded-For");

        if let Some(peer_addr) = origin_addr
            .and_then(|peer| peer.to_str().ok())
            .and_then(|addr| Ipv4Addr::from_str(addr).ok())
        {
            let should_allow = self
                .white_list
                .contain(ipset::convert_ipv4_addr_to_u32(&peer_addr.octets()));

            if should_allow {
                return Either::Left(self.service.call(req));
            }
        }
        Either::Right(ok(req.into_response(
            HttpResponse::Forbidden()
                .json(ApiError::new(CommonError::AddrNotSupported))
                .into_body(),
        )))
    }
}
