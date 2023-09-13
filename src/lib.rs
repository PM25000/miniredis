#![feature(impl_trait_in_assoc_type)]

pub struct S;

#[volo::async_trait]
impl volo_gen::miniredis::SlaveService for S {
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

#[volo::async_trait]
impl volo_gen::miniredis::MasterService for S {
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

#[volo::async_trait]
impl volo_gen::miniredis::ProxyService for S {
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
