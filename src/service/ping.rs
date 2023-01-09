
use crate::S;

use volo_gen::kite::ping::*;

#[volo::async_trait]
impl PingService for S {

    async fn ping(
        &self,
        _req: volo_grpc::Request<PingRequest>,
    ) -> Result<volo_grpc::Response<PongResponse>, volo_grpc::Status> {
        let request = _req.get_ref();

        Ok(volo_grpc::Response::new(PongResponse {
            text: request.text.clone(),
        }))
    }
}
