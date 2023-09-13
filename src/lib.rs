#![feature(impl_trait_in_assoc_type)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};
use volo::FastStr;
use volo_gen::miniredis;

lazy_static! {
    static ref GLOBAL_HASH_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub struct SlaveServiceS {
    pub addr: volo::net::Address,
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
}

#[volo::async_trait]
impl volo_gen::miniredis::MasterService for MasterServiceS {
    async fn set_item(
        &self,
        _request: volo_gen::miniredis::SetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SetItemResponse, ::volo_thrift::AnyhowError>
    {
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

        Ok(volo_gen::miniredis::SetItemResponse {
            message: String::from("Ok").into(),
        })
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
}

pub struct ProxyServiceS {
    pub master: Vec<(
        miniredis::MasterServiceClient,
        Vec<miniredis::SlaveServiceClient>,
    )>,
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
}
