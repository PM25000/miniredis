use lazy_static::lazy_static;
use pilota::FastStr;
use std::{
    io::{self, Write},
    net::SocketAddr, sync::Mutex,
};

lazy_static! {
    static ref CLIENT: volo_gen::miniredis::ProxyServiceClient = {
        let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
        volo_gen::miniredis::ProxyServiceClientBuilder::new("volo-example")
            // .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
    static ref TRANSACTION_ID: Mutex<Option<i64>> = Mutex::new(None);
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
        expire : None,
        transaction_id : *TRANSACTION_ID.lock().unwrap(),
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

async fn multi() -> volo_gen::miniredis::MultiResponse {
    if let Some(id) = *TRANSACTION_ID.lock().unwrap(){
        let resp = volo_gen::miniredis::MultiResponse {
            transaction_id : id
        };
        return resp;
    }
    let req = volo_gen::miniredis::MultiRequest {
        
    };
    let resp = CLIENT.multi(req).await;
    match resp {
        Ok(info) => {
            *TRANSACTION_ID.lock().unwrap() = Some(info.transaction_id);
            info
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn exec() -> volo_gen::miniredis::ExecResponse {
    if let None = *TRANSACTION_ID.lock().unwrap(){
        return Default::default();
    }
    let req = volo_gen::miniredis::ExecRequest {
        transaction_id : TRANSACTION_ID.lock().unwrap().unwrap(),
    };
    let resp = CLIENT.exec(req).await;
    *TRANSACTION_ID.lock().unwrap() = None;
    match resp {
        Ok(info) => {
            *TRANSACTION_ID.lock().unwrap() = None;
            info
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

async fn watch(key: FastStr) -> volo_gen::miniredis::WatchResponse {
    if let None = *TRANSACTION_ID.lock().unwrap(){
        return Default::default();
    }
    let req = volo_gen::miniredis::WatchRequest {
        key,
        transaction_id : TRANSACTION_ID.lock().unwrap().unwrap(),
    };
    let resp = CLIENT.watch(req).await;
    match resp {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("{:?}", e);
            Default::default()
        }
    }
}

// async fn ping(msg: Option<String>) -> volo_gen::miniredis::PingResponse {
//     let req = volo_gen::miniredis::PingRequest {
//         message: msg.map(|s| FastStr::from(s)),
//     };
//     let resp = CLIENT.ping(req).await;
//     match resp {
//         Ok(info) => info,
//         Err(e) => {
//             tracing::error!("{:?}", e);
//             Default::default()
//         }
//     }
// }


#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();


    let resp = set_item(FastStr::from("key"), FastStr::from("value")).await;

    assert_eq!(resp.message, FastStr::from("OK"));

    let resp = get_item(FastStr::from("key")).await;

    assert_eq!(resp.value, Some(FastStr::from("value")));

    for key in 0..100 {
        let resp = set_item(
            FastStr::from(format!("key{}", key)),
            FastStr::from(format!("value{}", key)),
        )
        .await;
        assert_eq!(resp.message, FastStr::from("OK"));
    }

    for key in 0..100 {
        let resp = get_item(FastStr::from(format!("key{}", key))).await;
        assert_eq!(
            resp.value,
            Some(FastStr::from(format!("value{}", key)))
        );
    }

    multi().await.transaction_id;

    let resp = set_item(FastStr::from("k1"), FastStr::from("v1")).await;

    assert_eq!(resp.message, FastStr::from("OK"));

    let resp = set_item(FastStr::from("k2"), FastStr::from("v2")).await;

    assert_eq!(resp.message, FastStr::from("OK"));

    let resp = get_item(FastStr::from("k1")).await;

    assert_eq!(resp.value, None);

    let resp = get_item(FastStr::from("k2")).await;

    assert_eq!(resp.value, None);

    exec().await;

    let resp = get_item(FastStr::from("k1")).await;

    assert_eq!(resp.value, Some(FastStr::from("v1")));

    let resp = get_item(FastStr::from("k2")).await;

    assert_eq!(resp.value, Some(FastStr::from("v2")));


}
