#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;

use mini_redis::S;
use volo_gen::miniredis;
#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::miniredis::SlaveServiceServer::new(S{master: Vec::new(), slave: Vec::new()})
        .run(addr)
        .await
        .unwrap();
}
