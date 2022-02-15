#![feature(test)]
extern crate test;
extern crate cogo_redis;

use test::Bencher;
use cogo_redis::cmd;
use cogo_redis::connector::RedisConnector;

#[bench]
fn bench_get(b: &mut Bencher) {
    let redis = RedisConnector::new("127.0.0.1:6379").connect().unwrap();
    redis.exec(cmd::Set("test", "value")).unwrap();

    b.iter(|| {
        redis.exec(cmd::Get("test"));
    });
}
