#![feature(type_alias_impl_trait)]

use std::net::SocketAddr;

use volo_grpc::server::{Server, ServiceBuilder};

use kite_server::S;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    Server::new()
        .add_service(ServiceBuilder::new(volo_gen::kite::ping::PingServiceServer::new(S)).build())
        .add_service(ServiceBuilder::new(volo_gen::kite::badge::BadgeServiceServer::new(S)).build())
        .run(addr)
        .await
        .unwrap();
}
