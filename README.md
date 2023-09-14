# miniredis
 miniredis

## AOF

## Redis 主从架构

### 配置文件
采用`json`文件来配置主节点的端口以及与之相连的从节点的`端口`.

文件位置为`/miniredis/master.config`,并且通过该配置文件,可以通过同级文件夹下的`start_m_s.py`脚本来一次性启动多个主节点和从节点.

### 架构实现
在`lib.rs`中实现了两个结构,分别代表`Master`和`Slave`.下面还列出了其函数表在`.thrift`文件中的表示.

`Master`多出的3个是为了实现`Transactions`.
```rust
pub struct SlaveServiceS {
    pub addr: volo::net::Address,//自身ip地址
    pub master: volo::net::Address,//所属Master的地址
}

pub struct MasterServiceS {
    pub slave: Vec<miniredis::SlaveServiceClient>,//slave-client
    pub addr: volo::net::Address,//自身地址
    pub rebuild: bool,//是否要通过AOF重建
}
```

```thrift
service MasterService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SetItemResponse SetItem(1: SetItemRequest request),
    DeleteItemResponse DeleteItem(1: DeleteItemRequest request),
    WatchResponse Watch(1: WatchRequest request),
    ServerMultiResponse ServerMulti(1: ServerMultiRequest request),
    ExecResponse Exec(1: ExecRequest request),
}

service SlaveService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SyncSetItemResponse SyncSetItem(1: SyncSetItemRequest request),
    SyncDeleteItemResponse SyncDeleteItem(1: SyncDeleteItemRequest request),
}

```
### 数据存储
主要采用一个`lazy_static`的全局变量来存储.

外层的`Mutex`是为了防止多个client访问而采用的互斥锁,内层的`HashMap`就是`KV`的存储.

该变量在主从节点中的作用是一样的
```rust
static ref GLOBAL_HASH_MAP: Mutex<HashMap<String, String>> = 
                                Mutex::new(HashMap::new());
```
### 从节点set返回错误
主要通过中间件来实现,这是主要的函数,其中通过判断`context`中`Endpoint`的数据,来判断是否为主节点的调用,如果是的话就继续推进,否则拒绝该次请求.

```rust
async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let callee = &cx.rpc_info().callee().unwrap().service_name;
        let req_msg=format!("{:?}",req);
        if req_msg.to_lowercase().contains("set"){
            tracing::info!("\n\n{:?}\n\n", callee);
            if !callee.contains("127.0.0.1:8080") {
                Err(anyhow!("Not master call").into())
            } else {
                self.0.call(cx, req).await
            }
        }else{
            self.0.call(cx, req).await
        }
    }
```

### 主从节点同步
而在调用主节点上的`set`命令时,会在该函数中调用`slave`成员的`set`命令,从而实现同步.
如下所示,(其中删去了部分代码来更加简洁)

```rust
//主节点的set_item函数中
...
for s in &self.slave {
    let resp = s
        .sync_set_item(...)
        .await;
}
...
```

### 测试
由于我们的项目是直接将`Cluster`和`主从`融合在一起,所以为了测试我只能额外写了个`client4ms_tst.rs`来进行请求.

于是我并没有实现该文件对于命令行输入的支持,而是直接在其主函数中调用相关函数.

启动方式为在`/miniredis`下,开启三个终端,按顺序执行.因为多是阻塞进程写脚本有点麻烦x.
1. cargo run --bin master 8080 8081
2. cargo run --bin slave 8081 8080
3. cargo run --bin client4ms_tst 8080 8081

里面实现了两个`client`,分别是`CLIENT_sla`与`CLIENT_mas`,第一个调用`从节点`的相关函数,第二个调用`主节点`.

主要执行了
1. set k1 111 (by `CLIENT_mas`)
2. set k1 111 (by `CLIENT_sla`)
3. get k1 (by `CLIENT_mas`)
4. get k1 (by `CLIENT_sla`)

输出为:
```
set Ok from CLIENT_mas
application error: service error, msg: Not master call
k1=111 from master
k1=111 from slave
```

第二条体现从节点不能`set`, 第四条体现主从节点共享数据

## Redis Cluster


