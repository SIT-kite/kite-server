mod service;

#[tokio::main]
async fn main() {

    tokio::join!(
        // Run grpc server
        service::grpc_server(),
    );
}