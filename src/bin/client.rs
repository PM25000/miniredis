use lazy_static::lazy_static;
use pilota::FastStr;
use volo_gen::miniredis;
use std::{
    io::{self, BufRead, Write},
    net::SocketAddr,
};

lazy_static! {
    static ref CLIENT: volo_gen::miniredis::ProxyServiceClient = {
        let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
        volo_gen::miniredis::ProxyServiceClientBuilder::new("volo-example")
            // .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}

async fn get_item(key: FastStr) -> volo_gen::miniredis::GetItemResponse {
    let req = volo_gen::miniredis::GetItemRequest { key };
    let resp = CLIENT.get_item(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
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

async fn delete_item(keys: Vec<FastStr>) -> volo_gen::miniredis::DeleteItemResponse {
    let req = volo_gen::miniredis::DeleteItemRequest { keys };
    let resp = CLIENT.delete_item(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn ping(msg: Option<String>) -> volo_gen::miniredis::PingResponse {
    let req = volo_gen::miniredis::PingRequest {
        message: msg.map(|s| FastStr::from(s)),
    };
    let resp = CLIENT.ping(req).await;
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
    tracing_subscriber::fmt::init();

    // let req = volo_gen::miniredis::PostItemRequest {
    //     name: FastStr::from("ddd"),
    // };
    // let resp = CLIENT.post_item(req).await;
    // match resp {
    //     Ok(info) => tracing::info!("{:?}", info),
    //     Err(e) => tracing::error!("{:?}", e),
    // }

    // let resp = set_item(FastStr::from("key"), FastStr::from("value")).await;

    // assert_eq!(resp.message, FastStr::from("OK"));

    // let resp = get_item(FastStr::from("key")).await;

    // assert_eq!(resp.value, FastStr::from("value"));

    // let resp = delete_item(vec![FastStr::from("key")]).await;

    // assert_eq!(resp.count, 1);

    // for i in 0..3 {
    //     let resp = ping(Some(format!("ping {}", i))).await;
    //     //sleep
    //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    //     tracing::info!("{:?}", resp);
    //     assert_eq!(resp.message, FastStr::from(format!("ping {}", i)));
    // }

    // let resp = ping(None).await;

    // assert_eq!(resp.message.to_ascii_lowercase(), FastStr::from("pong"));

    loop {
        print!("> ");
        io::stdout().flush().expect("failed to flush stdout");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read from stdin");

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let mut args = input.split_whitespace();
        let cmd = args.next().unwrap();
        let args = args.collect::<Vec<_>>();

        match cmd {
            "get" => {
                let key = args[0];
                let resp = get_item(String::from(key).into()).await;
                println!("{:?}", resp);
            }
            "set" => {
                let key = args[0];
                let value = args[1];
                let resp = set_item(String::from(key).into(), String::from(value).into()).await;
                println!("{:?}", resp);
            }
            "delete" => {
                let keys = args.iter().map(|s| String::from(*s).into()).collect();
                let resp = delete_item(keys).await;
                println!("{:?}", resp);
            }
            "ping" => {
                let msg = args.join(" ");
                let resp = if args.is_empty() {
                    ping(None).await
                } else {
                    ping(Some(msg)).await
                };
                println!("{:?}", resp);
            }
            "exit" => {
                break;
            }
            _ => {
                println!("unknown command: {}", cmd);
            }
        }
    }
}
