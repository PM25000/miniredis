#![feature(impl_trait_in_assoc_type)]

use std::hash::Hash;
use std::hash::Hasher;

use volo_gen::miniredis;

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

#[volo::async_trait]
impl volo_gen::miniredis::MasterService for MasterServiceS {
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
        let count = self.master.len();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let key = &_request.kv.key;
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let index = hash % count;
        let client = &self.master[index];
        let resp = client.set_item(_request).await;
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
        let count = self.slave.len()+self.master.len();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let key = &_request.key;
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let index = hash % count;
        let resp = if index < self.master.len() {
            let client = &self.master[index];
            client.get_item(_request).await
        }
        else {
            let client = &self.slave[index-self.master.len()];
            client.get_item(_request).await
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
