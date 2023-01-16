use tonic::{Request, Status};
use tonic::metadata::MetadataValue;
use tonic::transport::Server;

pub mod gen;

mod ping;


#[derive(Default)]
pub struct KiteGrpcServer {

}

pub async fn grpc_server() {
    let addr = "[::1]:50051".parse().unwrap();
    let server = KiteGrpcServer::default();

    let ping = ping::ping_service_server::PingServiceServer::new(server);

    Server::builder().add_service(ping).serve(addr).await.unwrap()
}
