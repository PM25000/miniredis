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

启动方式为在`/miniredis`下,开启两个终端,分别执行以下两个任务.第一行启动`Master+Slave`,第二行为`client`,并且命令已经写进去了

**注意 由于server端的特殊性,执行结束之后要手动杀死进程,否则可能后台占据端口**

```bash
python3 ./test4ms.py
cargo run --bin client4ms_tst 8080 8081
```

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

### 配置文件

采用`json`文件来配置集群中的各个节点的端口.

文件位置为`/miniredis/proxy.config`

格式为
```
[
    {
        "master": "127.0.0.1:8280",
        "slave": ["127.0.0.1:8281"]
    },
    {
        "master": "127.0.0.1:8380",
        "slave": ["127.0.0.1:8381"]
    },
    {
        "master": "127.0.0.1:8480",
        "slave": ["127.0.0.1:8481"]
    }
]
```

## Transaction


### Master part

主要实现三个函数`server_multi,watch,exec`.

#### 数据存储
该部分比较特殊的是使用到了两个全局变量

```rust
static ref GLOBAL_COMMAND_MAP: Mutex<HashMap<i64, HashMap<String, String>>> =
        Mutex::new(HashMap::new());
static ref GLOBAL_WATCHED_VALUE: Mutex<HashMap<i64, 
                        HashMap<String, Option<String>>>> 
                            =Mutex::new(HashMap::new());
```

1. GLOBAL_COMMAND_MAP: 主要用来记录在一个`transaction`的执行过程中那些量被修改了.外层HashMap的`key`是当前`事务ID`,用来区分各个事务的修改. 内层的HashMap则是被修改键值的`KV`

2. GLOBAL_WATCHED_VALUE: 主要记录被`watch`的值,大致结构与上文相似,除了最内层为`Option<String>`,这是为了区分在watch时尚未被设置的量,用`None`表示.

#### 执行

* server_multi\
    较为简单,就是初始化上述的两个全局变量.

* watch\
    也比较简单,取出当前键的值(若空设为`None`),并存到`GLOBAL_WATCHED_VALUE`中.

* exec\
    伪代码如下,主要逻辑为先检测`GLOBAL_WATCHED_VALUE`中的值与当前表中的值是否相同,若不同,返回错误.\
    接着遍历整个`GLOBAL_COMMAND_MAP`,将事务中被修改的值写入真实的表中.

```rust
for (kv:KV) in GLOBAL_WATCHED_VALUE{
    if changed
        return ERROR;
    else
        continue;
}
for (kv:KV) in GLOBAL_COMMAND_MAP{
    call self.set_it(kv)
}

```

## Proxy

主要实现所有函数的转发，以及 transaction id 的生成

当接受到`multi`命令时,会生成一个`transaction id`,并且将该`transaction id`返回给客户端.

其他命令携带的`transaction id`会被转发到对应的`master`节点或者`slave`节点.

### 命令实现

* get\
    通过`hash`函数来计算`key`所在的`master`节点或者`slave`节点,并且调用相应的函数.

* set\
    通过`hash`函数来计算`key`所在的`master`节点,并且调用相应的函数.

* multi\
    生成`transaction id`并返回

* watch\
    根据`key`计算出所在的`master`节点,并且调用相应的函数.

* exec\
    根据`transaction id`执行相关事务

### hash实现

```rust
        let count = self.master.len();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let key = &_request.key;
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let index = hash % count;
        let client = &self.master[index];

```

### proxy实现

```thrfit
service ProxyService {
    GetItemResponse GetItem(1: GetItemRequest request),
    SetItemResponse SetItem(1: SetItemRequest request),
    DeleteItemResponse DeleteItem(1: DeleteItemRequest request),
    WatchResponse Watch(1: WatchRequest request),
    MultiResponse Multi(1: MultiRequest request),
    ExecResponse Exec(1: ExecRequest request),
}
```

### 初始化实现

先读取配置文件，启动多个RPC服务，然后启动proxy服务

#### 存储

```rust
#[derive(Debug, Serialize, Deserialize)]
struct ProxyMaster {
    master: SocketAddr,
    slave: Vec<SocketAddr>,
}

struct ProxyTerminals {
    master: Vec<(miniredis::MasterServiceClient, Vec<miniredis::SlaveServiceClient>)>,
}

```

#### 初始化

```rust

    let mut index = 0;
    for master_item in data.into_iter() {
        let ProxyMaster { master: addr, slave } = master_item;
        let addr = volo::net::Address::from(addr);
        tracing::info!("master: {:?}", addr);
        let client = miniredis::MasterServiceClientBuilder::new(addr.to_string())
            .address(addr)
            .build();
        terminals.master.push((client, Vec::new()));
        for addr in slave {
            let addr = volo::net::Address::from(addr);
            tracing::info!("slave: {:?}", addr);
            let client = miniredis::SlaveServiceClientBuilder::new(addr.to_string())
                .address(addr)
                .build();
            terminals.master[index].1.push(client);
        }
        index += 1;
    }
    let ss = S {
        master: terminals.master,
    };


    let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::miniredis::ProxyServiceServer::new(ss)
        .run(addr)
        .await
        .unwrap();

    tracing::info!("Bye!");

```