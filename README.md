# co-redis
mco redis client

* Redis client for mco coroutine runtime


* example:
```toml
#Cargo.toml
mco-redis = "0.1"
```

```rust
use mco_redis::cmd;
use mco_redis::connector::RedisConnector;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let redis = RedisConnector::new("127.0.0.1:6379").connect()?;
    redis.exec(cmd::Set("test", "value"))?;
    if let Some(resp) = redis.exec(cmd::Get("test"))? {
        assert_eq!(resp, "value");
    }
    Ok(())
}
```
