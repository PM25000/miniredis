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

    let master_addr = String::from(&inargs[2]);
    let master_addr = format!("127.0.0.1:{}",master_addr);
    let master_addr:SocketAddr=master_addr.parse().unwrap();
    let master_addr=volo::net::Address::from(master_addr);

    let ss = SlaveServiceS{
        addr:slave_server.clone(),
        master: master_addr.clone(),
    };

    volo_gen::miniredis::SlaveServiceServer::new(ss)
        //.layer_front(mini_redis::ContextLayer)
        .run(slave_server)
        .await
        .unwrap();
    
}
