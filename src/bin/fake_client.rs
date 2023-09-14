use lazy_static::lazy_static;
use pilota::FastStr;
use std::fs::File;
use std::{
    io::{BufRead, BufReader, Read},
    net::SocketAddr,
};
use volo_gen::miniredis;

lazy_static! {
    static ref CLIENT: volo_gen::miniredis::ProxyServiceClient = {
        let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
        volo_gen::miniredis::ProxyServiceClientBuilder::new("volo-example")
            // .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}

async fn set_item(key: FastStr, value: FastStr) -> volo_gen::miniredis::SetItemResponse {
    let req = volo_gen::miniredis::SetItemRequest {
        kv: {
            let mut kv = volo_gen::miniredis::Kv::default();
            kv.key = key;
            kv.value = value;
            kv
        },
    };
    let resp = CLIENT.set_item(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

#[volo::main]
async fn main() {
    {
        let file = File::open("redis.aof").unwrap();
        // create a buffered reader
        let mut reader = BufReader::new(file);
        // iterate over the lines of the file

        loop {
            let mut input = reader.by_ref().lines().next().unwrap().unwrap();

            if input.is_empty() {
                continue;
            }

            let mut args = input.split_whitespace();
            let args = args.collect::<Vec<_>>();

            let key = args[0];
            let value = args[1];
            let resp = set_item(String::from(key).into(), String::from(value).into()).await;
            // println!("{:?}", resp);
        }
    }
}
