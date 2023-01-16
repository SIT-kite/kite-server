use tonic::Response;
pub use crate::service::gen::ping::*;

#[tonic::async_trait]
impl ping_service_server::PingService for super::KiteGrpcServer {
    async fn ping(&self, request: tonic::Request<PingRequest>) -> Result<tonic::Response<PongResponse>, tonic::Status> {
        let request = request.into_inner();

        Ok(Response::new(PongResponse {
            text: request.text
        }))
    }
}