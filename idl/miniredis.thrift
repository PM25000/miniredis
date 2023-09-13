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

service ProxyService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SetItemResponse SetItem(1: SetItemRequest request),
    DeleteItemResponse DeleteItem(1: DeleteItemRequest request),
}

service MasterService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SetItemResponse SetItem(1: SetItemRequest request),
    DeleteItemResponse DeleteItem(1: DeleteItemRequest request),
}

service SlaveService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SyncSetItemResponse SyncSetItem(1: SyncSetItemRequest request),
    SyncDeleteItemResponse SyncDeleteItem(1: SyncDeleteItemRequest request),
}
