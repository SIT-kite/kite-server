#![feature(type_alias_impl_trait)]

use volo_grpc::{Request, Response, Status};
use volo_gen::kite::ping::{PingRequest, PongResponse};

pub struct S;

#[volo::async_trait]
impl volo_gen::kite::ping::PingService for S {
    async fn ping(&self, req: Request<PingRequest>) -> Result<Response<PongResponse>, Status> {
        let request = req.into_inner();

        Ok(Response::new(PongResponse {
            text: request.text,
        }))
    }
}
