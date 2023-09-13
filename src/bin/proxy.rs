#![feature(impl_trait_in_assoc_type)]

use std::io::Read;
use std::{net::SocketAddr, fs::File};

use mini_redis::S; 
use serde::Serialize;
use serde::Deserialize;
use volo_gen::miniredis;

#[derive(Debug, Serialize, Deserialize)]
struct ProxyConfig {
    master: Vec<SocketAddr>,
    slave: Vec<SocketAddr>,
}
struct ProxyTerminals {
    master: Vec<miniredis::MasterServiceClient>,
    slave: Vec<miniredis::SlaveServiceClient>,
}

#[volo::main]
async fn main() {

    let mut settings = File::open("proxy.config").unwrap();
    let mut contents = String::new();
    settings.read_to_string(&mut contents).unwrap();
    tracing::info!("proxy.config: {}", contents);
    let data = serde_json::from_str::<ProxyConfig>(&contents).unwrap();
    let mut terminals = ProxyTerminals {
        master: Vec::new(),
        slave: Vec::new(),
    };

    for addr in data.master {
        let addr = volo::net::Address::from(addr);
        let client = miniredis::MasterServiceClientBuilder::new("volo-example")
            .address(addr)
            .build();
        terminals.master.push(client);
        tracing::info!("master: {:?}", addr);
    }
    for addr in data.slave {
        let addr = volo::net::Address::from(addr);
        let client = miniredis::SlaveServiceClientBuilder::new("volo-example")
            .address(addr)
            .build();
        terminals.slave.push(client);
        tracing::info!("slave: {:?}", addr);
    }

    let ss = S {
        master: terminals.master,
        slave: terminals.slave,
    };


    let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::miniredis::ProxyServiceServer::new(ss)
        .run(addr)
        .await
        .unwrap();
}
