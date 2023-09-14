#![feature(impl_trait_in_assoc_type)]

use anyhow::anyhow;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::format;
use volo::FastStr;
use volo_gen::miniredis;

lazy_static! {
    static ref GLOBAL_HASH_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref GLOBAL_COMMAND_MAP: Mutex<HashMap<i64, HashMap<String, String>>> =
        Mutex::new(HashMap::new());
    static ref GLOBAL_WATCHED_VALUE: Mutex<HashMap<i64, HashMap<String, Option<String>>>> =
        Mutex::new(HashMap::new());
}

pub struct SlaveServiceS {
    pub addr: volo::net::Address,
    pub master: volo::net::Address,
}

#[volo::async_trait]
impl volo_gen::miniredis::SlaveService for SlaveServiceS {
    async fn get_item(
        &self,
        _request: volo_gen::miniredis::GetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::GetItemResponse, ::volo_thrift::AnyhowError>
    {
        if let Some(value) = GLOBAL_HASH_MAP
            .lock()
            .unwrap()
            .get(&_request.key.to_string())
        {
            tracing::info!("get_item: {:?} in {:?}", _request, self.addr);
            return Ok(volo_gen::miniredis::GetItemResponse {
                value: Some(String::from(value).into()),
            });
        }
        tracing::info!("get_item: {:?} in {:?}", _request, self.addr);
        Ok(volo_gen::miniredis::GetItemResponse { value: None })
        // Ok(Default::default())
    }
    async fn sync_set_item(
        &self,
        _request: volo_gen::miniredis::SyncSetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SyncSetItemResponse, ::volo_thrift::AnyhowError>
    {
        // if external_variable != self.master{
        //     return Ok(volo_gen::miniredis::SyncSetItemResponse {
        //         message: String::from("Not master call").into(),
        //     });
        // }
        {
            GLOBAL_HASH_MAP
                .lock()
                .unwrap()
                .insert(_request.kv.key.to_string(), _request.kv.value.to_string());
        }
        tracing::info!("sync_set_item: {:?} in {:?}", _request, self.addr);

        Ok(volo_gen::miniredis::SyncSetItemResponse {
            message: String::from("OK").into(),
        })
        // Ok(Default::default())
    }
    async fn sync_delete_item(
        &self,
        _request: volo_gen::miniredis::SyncDeleteItemRequest,
    ) -> ::core::result::Result<
        volo_gen::miniredis::SyncDeleteItemResponse,
        ::volo_thrift::AnyhowError,
    > {
        Ok(Default::default())
    }
}

pub struct MasterServiceS {
    pub slave: Vec<miniredis::SlaveServiceClient>,
    pub addr: volo::net::Address,
    pub rebuild: bool,
}

#[volo::async_trait]
impl volo_gen::miniredis::MasterService for MasterServiceS {
    async fn set_item(
        &self,
        _request: volo_gen::miniredis::SetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SetItemResponse, ::volo_thrift::AnyhowError>
    {
        if let Some(id) = _request.transaction_id {
            tracing::info!("set_item: {:?} in {:?} with {:?} ", _request, self.addr, id);
            {
                GLOBAL_COMMAND_MAP
                    .lock()
                    .unwrap()
                    .entry(id)
                    .or_insert_with(|| HashMap::new())
                    .insert(_request.kv.key.to_string(), _request.kv.value.to_string());

                // .and_modify(|v| *v = _request.kv.value.to_string());
            }
            let tmp_t = GLOBAL_COMMAND_MAP.lock().unwrap();
            let tmp_t = tmp_t.get(&id);
            tracing::info!(
                "set_item: {:?} in {:?} with {:?} ",
                _request,
                self.addr,
                tmp_t
            );
            Ok(volo_gen::miniredis::SetItemResponse {
                message: String::from("Ok").into(),
            })
        } else {
            {
                GLOBAL_HASH_MAP
                    .lock()
                    .unwrap()
                    .insert(_request.kv.key.to_string(), _request.kv.value.to_string());
            }
            for s in &self.slave {
                let resp = s
                    .sync_set_item(volo_gen::miniredis::SyncSetItemRequest {
                        kv: volo_gen::miniredis::Kv {
                            key: _request.kv.key.clone(),
                            value: _request.kv.value.clone(),
                        },
                    })
                    .await;
                match resp {
                    Ok(_resp) => {
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            tracing::info!("set_item: {:?} in {:?}", _request, self.addr);

            if !(self.rebuild) {
                let protocol_text = format!(
                    "{} {}\r\n",
                    _request.kv.key.to_string(),
                    _request.kv.value.to_string()
                );
                // convert the protocol text to bytes
                let protocol_bytes = protocol_text.as_bytes();
    
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open("redis.aof")?;
                // 向文件中写入一个SET命令
                std::io::Write::write_all(&mut file, protocol_bytes).unwrap();
            }

            Ok(volo_gen::miniredis::SetItemResponse {
                message: String::from("Ok").into(),
            })
        }
    }
    async fn delete_item(
        &self,
        _request: volo_gen::miniredis::DeleteItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::DeleteItemResponse, ::volo_thrift::AnyhowError>
    {
        Ok(Default::default())
    }
    async fn get_item(
        &self,
        _request: volo_gen::miniredis::GetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::GetItemResponse, ::volo_thrift::AnyhowError>
    {
        if let Some(value) = GLOBAL_HASH_MAP
            .lock()
            .unwrap()
            .get(&_request.key.to_string())
        {
            tracing::info!("get_item: {:?} in {:?}", _request, self.addr);
            return Ok(volo_gen::miniredis::GetItemResponse {
                value: Some(String::from(value).into()),
            });
        }
        tracing::info!("get_item: {:?} in {:?}", _request, self.addr);
        Ok(volo_gen::miniredis::GetItemResponse { value: None })
    }

    async fn server_multi(
        &self,
        _request: volo_gen::miniredis::ServerMultiRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::ServerMultiResponse, ::volo_thrift::AnyhowError>
    {
        GLOBAL_COMMAND_MAP
            .lock()
            .unwrap()
            .insert(_request.transaction_id, HashMap::new());
        GLOBAL_WATCHED_VALUE
            .lock()
            .unwrap()
            .insert(_request.transaction_id, HashMap::new());
        Ok(volo_gen::miniredis::ServerMultiResponse {
            message: "Get into transaction!".to_string().into(),
        })
        // Ok(Default::default())
    }

    async fn exec(
        &self,
        _request: volo_gen::miniredis::ExecRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::ExecResponse, ::volo_thrift::AnyhowError> {
        
        {
            tracing::info!("exec: {:?} in {:?}", _request, self.addr);
            let local_watch_t = GLOBAL_WATCHED_VALUE.lock().unwrap();
            let inner = local_watch_t.get(&_request.transaction_id).unwrap();
            let globla_t = GLOBAL_HASH_MAP.lock().unwrap();
            for (key, op_value) in inner.iter() {
                if let Some(new_value) = op_value {
                    if &globla_t[key] == new_value {
                        continue;
                    } else {
                        let _ = local_watch_t.get(&_request.transaction_id).clone();
                        let msg = format!("{} was been changed", key);
                        return Err(anyhow::Error::msg(msg).into());
                    }
                } else {
                    if globla_t.contains_key(key) {
                        let _ = local_watch_t.get(&_request.transaction_id).clone();
                        let msg = format!("{} was been changed", key);
                        return Err(anyhow::Error::msg(msg).into());
                    } else {
                        continue;
                    }
                }
            }
            let _ = local_watch_t.get(&_request.transaction_id).clone();
        }

        {
            let mut inner: HashMap<String, String> = HashMap::new();
            {
                let mut local_set = GLOBAL_COMMAND_MAP.lock().unwrap();
                if let Some(inner_map) = local_set.get_mut(&_request.transaction_id) {
                    // 复制最内层的 HashMap。
                    inner = inner_map.clone();

                    // 删除最内层的 HashMap。
                    local_set.remove(&_request.transaction_id);
                }
            }
            tracing::info!("inner {:?}", inner);
            for (key, value) in inner.iter() {
                self.set_item(volo_gen::miniredis::SetItemRequest {
                    kv: volo_gen::miniredis::Kv {
                        key: key.to_string().into(),
                        value: value.to_string().into(),
                    },
                    expire: None,
                    transaction_id: None,
                }).await?;
                // {
                //     GLOBAL_HASH_MAP
                //         .lock()
                //         .unwrap()
                //         .insert(key.to_string(), value.to_string());
                // }
                // for s in &self.slave {
                //     let resp = s
                //         .sync_set_item(volo_gen::miniredis::SyncSetItemRequest {
                //             kv: volo_gen::miniredis::Kv {
                //                 key: key.to_string().into(),
                //                 value: value.to_string().into(),
                //             },
                //         })
                //         .await;
                //     match resp {
                //         Ok(_resp) => {
                //             continue;
                //         }
                //         Err(err) => return Err(err.into()),
                //     }
                // }
            }
            // let _= local_set.get(&_request.transaction_id).clone();
        }

        Ok(volo_gen::miniredis::ExecResponse {
            message: "EXEC successfully".into(),
        })

        // Ok(Default::default())
    }

    async fn watch(
        &self,
        _request: volo_gen::miniredis::WatchRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::WatchResponse, ::volo_thrift::AnyhowError>
    {
        {
            if let Some(value) = GLOBAL_HASH_MAP
                .lock()
                .unwrap()
                .get(&_request.key.to_string())
            {
                // let local_watchT=GLOBAL_WATCHED_VALUE.lock().unwrap().get
                GLOBAL_WATCHED_VALUE
                    .lock()
                    .unwrap()
                    .entry(_request.transaction_id)
                    .or_insert_with(|| HashMap::new())
                    .insert(_request.key.to_string(), Some(value.clone()));
                    // .entry(_request.key.to_string())
                    // .and_modify(|v| *v = Some(value.clone()));
                // .get(&_request.transaction_id)
                // .unwrap()
                // .insert(_request.key.to_string().clone(),Some(value.clone()));
            } else {
                GLOBAL_WATCHED_VALUE
                    .lock()
                    .unwrap()
                    .entry(_request.transaction_id)
                    .or_insert_with(|| HashMap::new())
                    .insert(_request.key.to_string(), None);
                    // .entry(_request.key.to_string())
                    // .and_modify(|v| *v = None);
            }
        }
        Ok(volo_gen::miniredis::WatchResponse {
            message: "Watch has been set".into(),
        })
    }
}

pub struct ProxyServiceS {
    pub master: Vec<(
        miniredis::MasterServiceClient,
        Vec<miniredis::SlaveServiceClient>,
    )>,
}

lazy_static! {
    static ref TRANSACTION_ID: Mutex<i64> = Mutex::new(0);
}

#[volo::async_trait]
impl volo_gen::miniredis::ProxyService for ProxyServiceS {
    async fn set_item(
        &self,
        _request: volo_gen::miniredis::SetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SetItemResponse, ::volo_thrift::AnyhowError>
    {
        let count = self.master.len();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let key = &_request.kv.key;
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let index = hash % count;
        let client = &self.master[index];
        let resp = client.0.set_item(_request).await;
        match resp {
            Ok(info) => {
                tracing::info!("set_item: {:?} in {:?}", info, index);
                Ok(info)
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(::volo_thrift::AnyhowError::from(e))
            }
        }
    }
    async fn delete_item(
        &self,
        _request: volo_gen::miniredis::DeleteItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::DeleteItemResponse, ::volo_thrift::AnyhowError>
    {
        Ok(Default::default())
    }
    async fn get_item(
        &self,
        _request: volo_gen::miniredis::GetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::GetItemResponse, ::volo_thrift::AnyhowError>
    {
        let count = self.master.len();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let key = &_request.key;
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let index = hash % count;
        let client = &self.master[index];
        let count = client.1.len() + 1;
        let masterindex = index;
        let index = hash % count;
        let resp = if index == 0 {
            client.0.get_item(_request).await
        } else {
            client.1[index - 1].get_item(_request).await
        };

        match resp {
            Ok(info) => {
                tracing::info!("get_item: {:?} in {:?} in {:?}", info, masterindex, index);
                Ok(info)
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(::volo_thrift::AnyhowError::from(e))
            }
        }
    }

    async fn watch(
        &self,
        _request: volo_gen::miniredis::WatchRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::WatchResponse, ::volo_thrift::AnyhowError>
    {
        let count = self.master.len();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let key = &_request.key;
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let index = hash % count;
        let client = &self.master[index];
        let resp = client.0.watch(_request).await;
        match resp {
            Ok(info) => {
                tracing::info!("watch: {:?} in {:?}", info, index);
                Ok(info)
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(::volo_thrift::AnyhowError::from(e))
            }
        }
    }

    async fn multi(
        &self,
        _request: volo_gen::miniredis::MultiRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::MultiResponse, ::volo_thrift::AnyhowError>
    {
        let new_id = {
            let mut transaction_id = TRANSACTION_ID.lock().unwrap();
            *transaction_id += 1;
            *transaction_id
        };
        let mut index = 0;
        for client in &self.master {
            let resp = client
                .0
                .server_multi(volo_gen::miniredis::ServerMultiRequest {
                    transaction_id: new_id,
                })
                .await;
            match resp {
                Ok(info) => {
                    tracing::info!("multi: {:?} in {:?}", info, index);
                }
                Err(e) => {
                    tracing::error!("{:?}", e);
                }
            };
            index += 1;
        }
        Ok(volo_gen::miniredis::MultiResponse {
            transaction_id: new_id,
        })
    }

    async fn exec(
        &self,
        _request: volo_gen::miniredis::ExecRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::ExecResponse, ::volo_thrift::AnyhowError> {
        let mut index = 0;
        let mut err_flag = false;
        for client in &self.master {
            let resp = client.0.exec(_request.clone()).await;
            match resp {
                Ok(info) => {
                    tracing::info!("exec: {:?} in {:?}", info, index);
                }
                Err(e) => {
                    tracing::error!("{:?}", e);
                    err_flag = true;
                }
            };
            index += 1;
        }
        if err_flag {
            Err(anyhow!("EXEC failed").into())
        } else {
            Ok(volo_gen::miniredis::ExecResponse {
                message: "EXEC successfully".into(),
            })
        }
    }
}

#[derive(Clone)]
pub struct ContextService<S>(S);

#[volo::service]
impl<Req, S, Cx> volo::Service<Cx, Req> for ContextService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static + volo::context::Context,
    anyhow::Error: Into<S::Error>,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        // println!("\n\nin layer\n\n");
        // tracing_subscriber::fmt::init();
        let callee = &cx.rpc_info().callee().unwrap().service_name;
        // let caller=&cx.rpc_info().caller().unwrap();
        // let caller=format!("{:?}",caller);
        // tracing::info!("\n\n{:?}\n\n",callee);
        tracing::info!("\n\n{:?}\n\n", callee);
        if !callee.contains("127.0.0.1:8080") {
            Err(anyhow!("Not master call").into())
        } else {
            self.0.call(cx, req).await
        }
    }
}

pub struct ContextLayer;

impl<S> volo::Layer<S> for ContextLayer {
    type Service = ContextService<S>; //

    fn layer(self, inner: S) -> Self::Service {
        ContextService(inner)
    }
}
