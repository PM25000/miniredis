#![feature(impl_trait_in_assoc_type)]

//cargo run --bin  <slavePort>
use std::net::SocketAddr;
// use mini_redis::SlaveServiceS;
#[volo::main]

async fn main(){
    let inargs: Vec<String> = std::env::args().collect();

    let addr_mas=String::from(&inargs[1]);
    let addr_mas = format!("127.0.0.1:{}",addr_mas);
    let addr_mas :SocketAddr=addr_mas.parse().unwrap();

    let CLIENT_mas=volo_gen::miniredis::MasterServiceClientBuilder::new("tst_client_mas")
                                .address(addr_mas)
                                .build();
    
    let addr_sla=String::from(&inargs[2]);
    let addr_sla = format!("127.0.0.1:{}",addr_sla);
    let addr_sla :SocketAddr=addr_sla.parse().unwrap();

    let CLIENT_sla =volo_gen::miniredis::SlaveServiceClientBuilder::new("tst_client_sla")
                                .layer_outer(mini_redis::ContextLayer)
                                .address(addr_sla)
                                .build();

    let resp= CLIENT_mas.set_item(volo_gen::miniredis::SetItemRequest{
        kv:volo_gen::miniredis::Kv{
            key: "k1".into(),
            value: "111".into(),
        },
        expire:None,
        transaction_id:None,
    }).await;
    match resp{
        Ok(msg) =>println!("set {} from CLIENT_mas",msg.message),
        Err(err) => println!("{}",err),
    }

    let resp= CLIENT_sla.sync_set_item(volo_gen::miniredis::SyncSetItemRequest{
        kv:volo_gen::miniredis::Kv{
            key: "k2".into(),
            value: "111".into(),
        }
    }).await;
    match resp{
        Ok(msg) =>println!("{}",msg.message),
        Err(err) => println!("{}",err),
    }

    let resp= CLIENT_mas.get_item(volo_gen::miniredis::GetItemRequest{
        key: "k1".into()
    }).await;
    match resp {
        Ok(resp) => {
            if let Some(value)=resp.value{
                println!("k1={} from master", value);
            }else{
                println!("k1=(nil) from master");
            }
        },
        Err(err)=>println!("Get Failed: {}", err),
    }
    

    let resp= CLIENT_sla.get_item(volo_gen::miniredis::GetItemRequest{
        key: "k1".into()
    }).await;
    match resp {
        Ok(resp) => {
            if let Some(value)=resp.value{
                println!("k1={} from slave", value);
            }else{
                println!("k1=(nil) from slave");
            }
        },
        Err(err)=>println!("Get Failed: {}", err),
    }
}