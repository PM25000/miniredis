namespace rs miniredis

struct KV {
    1: required string key,
    2: required string value,
}

struct GetItemRequest {
    1: required string key,
}

struct GetItemResponse {
    1: optional string value,
}

struct SetItemRequest {
    1: required KV kv,
    2: optional i64 expire,
    3: optional i64 transactionId,
}

struct SetItemResponse {
    1: required string message,
}

struct DeleteItemRequest {
    1: required list<string> keys,
}

struct DeleteItemResponse {
    1: required i64 count,
}

struct SyncSetItemRequest {
    1: required KV kv,
}

struct SyncSetItemResponse {
    1: required string message,
}

struct SyncDeleteItemRequest {
    1: required list<string> keys,
}

struct SyncDeleteItemResponse {
    1: required i64 count,
}

struct WatchRequest {
    1: required string key,
    2: required i64 transactionId,
}

struct WatchResponse {
    1: required string message,
}

struct MultiRequest {
    
}

struct MultiResponse {
    1: required i64 transactionId,
}

struct ExecRequest {
    1: required i64 transactionId,
}

struct ExecResponse {
    1: required string message,
}

service ProxyService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SetItemResponse SetItem(1: SetItemRequest request),
    DeleteItemResponse DeleteItem(1: DeleteItemRequest request),
    WatchResponse Watch(1: WatchRequest request),
    MultiResponse Multi(1: MultiRequest request),
    ExecResponse Exec(1: ExecRequest request),
}

service MasterService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SetItemResponse SetItem(1: SetItemRequest request),
    DeleteItemResponse DeleteItem(1: DeleteItemRequest request),
    WatchResponse Watch(1: WatchRequest request),
    MultiResponse Multi(1: MultiRequest request),
    ExecResponse Exec(1: ExecRequest request),
}

service SlaveService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SyncSetItemResponse SyncSetItem(1: SyncSetItemRequest request),
    SyncDeleteItemResponse SyncDeleteItem(1: SyncDeleteItemRequest request),
}
