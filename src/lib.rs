#![feature(impl_trait_in_assoc_type)]
use lazy_static::lazy_static;
use std::{collections::HashMap,
		  sync::Mutex
		  };
use volo_gen::miniredis;

lazy_static! {
	static ref GLOBAL_HASH_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub struct SlaveServiceS;

#[volo::async_trait]
impl volo_gen::miniredis::SlaveService for SlaveServiceS {
    async fn get_item(
        &self,
        _request: volo_gen::miniredis::GetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::GetItemResponse, ::volo_thrift::AnyhowError>
    {
        if let Some(value)=GLOBAL_HASH_MAP
                                .lock()
                                .unwrap()
                                .get(&_request.key.to_string())
        {
            return Ok(volo_gen::miniredis::GetItemResponse{value:Some(String::from(value).into())});
        }
        Ok(volo_gen::miniredis::GetItemResponse{value:None})
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
                    .insert(_request.kv.key.to_string(),_request.kv.value.to_string());
        }
        Ok(volo_gen::miniredis::SyncSetItemResponse{message: String::from("OK").into()})
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

pub struct MasterServiceS{
    pub slave: Vec<miniredis::SlaveServiceClient>,
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
                        .insert(_request.kv.key.to_string(),_request.kv.value.to_string());
        }
        for s in &self.slave{
            let resp=s.sync_set_item(
                volo_gen::miniredis::SyncSetItemRequest{
                    kv: volo_gen::miniredis::Kv{
                        key: _request.kv.key.clone(),
                        value: _request.kv.value.clone(),
                    }
                }
            ).await;
            match resp{
                Ok(_resp)=>{
                    continue;
                },
                Err(err) => return Err(err.into()),
            }
        }
        Ok(volo_gen::miniredis::SetItemResponse{message: String::from("Ok").into()})
        
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
        if let Some(value)=GLOBAL_HASH_MAP
                                .lock()
                                .unwrap()
                                .get(&_request.key.to_string())
        {
            return Ok(volo_gen::miniredis::GetItemResponse{value:Some(String::from(value).into())});
        }
        Ok(volo_gen::miniredis::GetItemResponse{value:None})
    }
}

pub struct ProxyServiceS {
    pub master: Vec<miniredis::MasterServiceClient>,
    pub slave: Vec<miniredis::SlaveServiceClient>,
}

#[volo::async_trait]
impl volo_gen::miniredis::ProxyService for ProxyServiceS {
    async fn set_item(
        &self,
        _request: volo_gen::miniredis::SetItemRequest,
    ) -> ::core::result::Result<volo_gen::miniredis::SetItemResponse, ::volo_thrift::AnyhowError>
    {
        Ok(Default::default())
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
        Ok(Default::default())
    }
}
