#![feature(impl_trait_in_assoc_type)]

//cargo run --bin master <MasterPort> <SlavePort> [SlavePort...]

use mini_redis::MasterServiceS;
use std::default;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::net::SocketAddr;
use volo::FastStr;
use volo_gen::miniredis::{Kv, MasterService};

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // let addr: SocketAddr = "[::]:8080".parse().unwrap();
    // let addr = volo::net::Address::from(addr);
    let inargs: Vec<String> = std::env::args().collect();

    // let addr_slave = volo::net::Address::from(addr_slave);

    let addr_proxy = String::from(&inargs[1]);
    let addr_proxy_str = format!("127.0.0.1:{}", addr_proxy);
    let addr_proxy: SocketAddr = addr_proxy_str.parse().unwrap();
    let addr_proxy = volo::net::Address::from(addr_proxy);

    let mut slaves = Vec::<volo_gen::miniredis::SlaveServiceClient>::new();
    for i in 2..inargs.len() {
        let addr_slave = String::from(&inargs[i]);
        let addr_slave = format!("127.0.0.1:{}", addr_slave);
        let addr_slave: SocketAddr = addr_slave.parse().unwrap();

        // let client_of_slave: volo_gen::miniredis::SlaveServiceClient = {
        //     volo_gen::miniredis::SlaveServiceClientBuilder::new(addr_proxy_str.to_string())
        //         .address(addr_slave)
        //         .build()
        // };
        slaves.push({
            volo_gen::miniredis::SlaveServiceClientBuilder::new(addr_proxy_str.to_string())
                .address(addr_slave)
                .build()
        });
    }

    let master = MasterServiceS {
        slave: slaves,
        addr: addr_proxy.clone(),
    };

    let mut file = File::open("redis.aof").unwrap();
    let reader = BufReader::new(&mut file);

    for line in reader.lines() {
        let line = line.unwrap().clone();

        if line.is_empty() {
            break;
        }

        let mut args = line.split_whitespace();

        let key: FastStr = FastStr::from(args.next().unwrap());
        let value: FastStr = FastStr::from(args.next().unwrap());

        let kk = Kv {
            key: key,
            value: value,
        };
        let req = volo_gen::miniredis::SetItemRequest { kv: kk };
        let _resp = volo_gen::miniredis::MasterService::set_item(&master, req).await;
        // println!("{:?}", resp);
    }

    volo_gen::miniredis::MasterServiceServer::new(master)
        .run(addr_proxy)
        .await
        .unwrap();
}
