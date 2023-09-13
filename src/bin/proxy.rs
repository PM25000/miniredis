#![feature(impl_trait_in_assoc_type)]

use std::io::Read;
use std::{net::SocketAddr, fs::File};

use mini_redis::ProxyServiceS as S; 
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

    tracing_subscriber::fmt::init();

    let mut settings = File::open("proxy.config").unwrap();
    let mut contents = String::new();

    settings.read_to_string(&mut contents).unwrap();
    
    tracing::info!("proxy.config: {}", contents);
    // println!("proxy.config: {}", contents);

    let data = serde_json::from_str::<ProxyConfig>(&contents).unwrap();
    let mut terminals = ProxyTerminals {
        master: Vec::new(),
        slave: Vec::new(),
    };

    for addr in data.master {
        let addr = volo::net::Address::from(addr);
        tracing::info!("master: {:?}", addr);
        let client = miniredis::MasterServiceClientBuilder::new(addr.to_string())
            .address(addr)
            .build();
        terminals.master.push(client);
    }
    for addr in data.slave {
        let addr = volo::net::Address::from(addr);
        tracing::info!("slave: {:?}", addr);
        let client = miniredis::SlaveServiceClientBuilder::new(addr.to_string())
            .address(addr)
            .build();
        terminals.slave.push(client);
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
