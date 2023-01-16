use tonic::{Request, Response, Status};

pub use crate::service::gen::ping as gen;

#[tonic::async_trait]
impl gen::ping_service_server::PingService for super::KiteGrpcServer {
    async fn ping(&self, request: Request<gen::PingRequest>) -> Result<Response<gen::PongResponse>, Status> {
        let request = request.into_inner();

        Ok(Response::new(gen::PongResponse { text: request.text }))
    }
}
