#![feature(impl_trait_in_assoc_type)]

//cargo run --bin  <slavePort>
use std::net::SocketAddr;
use mini_redis::SlaveServiceS;
#[volo::main]

async fn main() {
    tracing_subscriber::fmt::init();
    
    let inargs: Vec<String> = std::env::args().collect();

    let slave_server = String::from(&inargs[1]);
    let slave_server = format!("127.0.0.1:{}",slave_server);
    let slave_server:SocketAddr=slave_server.parse().unwrap();
    let slave_server=volo::net::Address::from(slave_server);

    let ss = SlaveServiceS{
        addr:slave_server.clone(),
    };

    volo_gen::miniredis::SlaveServiceServer::new(ss)
        .run(slave_server)
        .await
        .unwrap();
    
}
