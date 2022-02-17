use mco_redis::cmd;
use mco_redis::connector::RedisConnector;
use std::error::Error;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // or use connect_timeout(Duration::from_secs(30))
    let redis = RedisConnector::new("127.0.0.1:6379").connect()?;
    redis.exec(cmd::Set("test", "value"))?;
    if let Some(resp) = redis.exec(cmd::Get("test"))? {
        assert_eq!(resp, "value");
    }
    Ok(())
}