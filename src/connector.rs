use std::io;
use std::net::ToSocketAddrs;
use cogo::net::TcpStream;
use crate::bytes::{ByteString};
use crate::client::Client;
use crate::cmd;
use crate::simple::SimpleClient;
use super::errors::ConnectError;

/// Redis connector
pub struct RedisConnector<A> {
    address: A,
    passwords: Vec<ByteString>,
}

impl<A> RedisConnector<A>
    where
        A: ToSocketAddrs + Clone,
{
    /// Create new redis connector
    pub fn new(address: A) -> RedisConnector<A> {
        RedisConnector {
            address: address.clone(),
            passwords: Vec::new(),
        }
    }
}

impl<A> RedisConnector<A>
    where
        A: ToSocketAddrs + Clone,
{
    /// Add redis auth password
    pub fn password<U>(mut self, password: U) -> Self
        where
            U: AsRef<str>,
    {
        self.passwords.push(ByteString::from(password.as_ref().to_string()));
        self
    }

    /// Use custom connector
    pub fn connector(self) -> RedisConnector<A> where IoBoxed: From<U::Response> {
        RedisConnector {
            address: self.address,
            passwords: self.passwords,
        }
    }
}

impl<A> RedisConnector<A>
    where
        A: ToSocketAddrs + Clone,
{
    fn _connect(&mut self) -> Result<SimpleClient, ConnectError> {
        let passwords = self.passwords.clone();
        let conn = TcpStream::connect(self.address.clone())?;
        // let io = IoBoxed::from(fut.await?);
        // io.set_memory_pool(pool);
        // io.set_disconnect_timeout(Seconds::ZERO.into());
        if passwords.is_empty() {
            Ok(io)
        } else {
            let client = SimpleClient::new(conn);
            for password in passwords {
                if client.exec(cmd::Auth(password)).await? {
                    return Ok(client);
                }
            }
            self.connector = None;
            Err(ConnectError::Unauthorized)
        }
    }

    /// Connect to redis server and create shared client
    pub fn connect(&mut self) -> Result<Client, ConnectError> {
        Ok(Client::new(self._connect()?))
    }

    /// Connect to redis server and create simple client
    pub fn connect_simple(&mut self) -> Result<SimpleClient, ConnectError> {
        Ok(self._connect()?)
    }
}
