use crate::ipset;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use futures::future::{ok, Either, Ready};
use std::net::IpAddr;
use std::task::{Context, Poll};

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
        // Get peer's IP addr. But I'm confused that, when does the peer_addr() be None? When unit test?
        // Maybe!
        if let Some(peer_addr) = req.peer_addr() {
            let should_allow = match peer_addr.ip() {
                // Whether IPv4 addr is in white list.
                IpAddr::V4(addr) => self
                    .white_list
                    .contain(ipset::convert_ipv4_addr_to_u32(&addr.octets())),
                // We don't have IPv6 environment, return true by default.
                IpAddr::V6(_) => true,
            };
            if should_allow {
                return Either::Left(self.service.call(req));
            }
            return Either::Right(ok(req.into_response(
                HttpResponse::Forbidden()
                    .json(r#"{"code": 503, "msg": "不支持您所在的区域"}"#)
                    .into_body(),
            )));
        }
        // When unit test?
        return Either::Left(self.service.call(req));
    }
}
