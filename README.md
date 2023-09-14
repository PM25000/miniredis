# miniredis
## 1.AOF(Append-only File)
AOF is a persistence method that records every write operation received by the Redis server in an append-only file. This way, the Redis server can reconstruct its original dataset by replaying the commands in the AOF file when it restarts .

So the main process of AOF contains two parts: first write the operation into aof file and then reconstruct the dataset by replaying the commands in the aof file.

In fact, in redis, AOF is much more complex and it also contains a method of rewrite aof file when it's too big and takes too much time to rebuild. However, the size of our commands is not that large, so we didn't implement rewriting aof.

First, the write-in process is implemented in lib.rs, in the body of MasterService, in the get_item function.
```rust
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
```
The "rebuild" we will talk later, because we only need to care about set operation, we don't need to store the command but just the key and value. Put key and value into string and append it tothe aof file and become a new line. When reconstructing, we just need to read line by line and replay the set operations. 

Then the reconstruction is in master.rs, in the main function. After initialize the MasterServices, use the entility to call the set_item function to replay the operations.
```rust
let mut file = File::open("redis.aof").unwrap();
    let reader = BufReader::new(&mut file);

    for line in reader.lines() {
        let line = line.unwrap().clone();

        if line.is_empty() {
            break;
        }

        let mut args = line.split_whitespace();

        let kk = Kv {
            key: String::from(args.next().unwrap()).into(),
            value: String::from(args.next().unwrap()).into(),
        };
        let req = volo_gen::miniredis::SetItemRequest { kv: kk };
        let _resp = volo_gen::miniredis::MasterService::set_item(
            &master, req).await;
        // println!("{:?}", resp);
    }
``````
Open the redis.aof file and use BufReader to read the file line by line. Every line has a key and a value, so just put the key and value into the request and call the set_item function to replay operations.

However, I met a problem, when reconstructing the dataset, we call the set_item, but get_item will write the operation into aof again! So the aof will never be read over, the servers will infinitely repeat set operations.

To avoid such situation, we adjust the struct of the MasterServices, add a rebuild bool to tell the function whether the server is reconstructed. If rebuild is false, it's normal mode, just write the operations into aof. But if rebuild is true, server mustn't write into aof.
So before the reconstruction, the initialization of MasterServices is 
```rust
let mut master = MasterServiceS {
        slave: slaves,
        addr: addr_proxy.clone(),
        rebuild: true,
    };
```
set rebuild as true.
But after reconstruction, we add a line:
```rust
master.rebuild = false;
```
turn rebuild into false, show that it's normal mode.
___




## 2.Master-Slave Structure



## 3.Redis Cluster
