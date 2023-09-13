#![feature(impl_trait_in_assoc_type)]

use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use volo_gen::miniredis;
use lazy_static::lazy_static;
use volo::FastStr;
pub struct SlaveServiceS;

#[volo::async_trait]
impl volo_gen::miniredis::SlaveService for SlaveServiceS {
    async fn get_item(
        &self,
        _request: volo_gen::miniredis::GetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::GetItemResponse, ::volo_thrift::AnyhowError>
    {
        Ok(Default::default())
    }
    async fn sync_set_item(
        &self,
        _request: volo_gen::miniredis::SyncSetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SyncSetItemResponse, ::volo_thrift::AnyhowError>
    {
        Ok(Default::default())
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

pub struct MasterServiceS;

type Db = Arc<Mutex<HashMap<FastStr, FastStr>>>;
lazy_static! {
    static ref DB: Db = Arc::new(Mutex::new(HashMap::new()));
}

#[volo::async_trait]
impl volo_gen::miniredis::MasterService for MasterServiceS {
    async fn set_item(
        &self,
        _request: volo_gen::miniredis::SetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SetItemResponse, ::volo_thrift::AnyhowError>
    {
        println!("set_item");
        println!("{}:{}", _request.kv.key.to_string(), _request.kv.value.to_string());
        let mut db = DB.lock().unwrap();
        db.insert(_request.kv.key, _request.kv.value);
        Ok(volo_gen::miniredis::SetItemResponse {
            message: FastStr::from("OK"),
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
        println!("get_item");
        println!("{}", _request.key.to_string());
        let db = DB.lock().unwrap();
        let value = db.get(&_request.key);
        match value {
            Some(v) => {
                let mut resp = volo_gen::miniredis::GetItemResponse::default();
                resp.value = Some(v.clone());
                Ok(resp)
            }
            None =>{
                let mut resp = volo_gen::miniredis::GetItemResponse::default();
                resp.value = None;
                Ok(resp)
            }
        }
    }
}

pub struct ProxyServiceS {
    pub master: Vec<(miniredis::MasterServiceClient, Vec<miniredis::SlaveServiceClient>)>,
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
                tracing::info!("set_item: {:?}", info);
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
        let count = client.1.len()+1;
        let index = hash % count;
        let resp = if index == 0 {
            client.0.get_item(_request).await
        } else {
            client.1[index-1].get_item(_request).await
        };
        
        match resp {
            Ok(info) => {
                tracing::info!("get_item: {:?}", info);
                Ok(info)
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(::volo_thrift::AnyhowError::from(e))
            }
        }
    }
}
