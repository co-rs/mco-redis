use std::io;
use std::net::ToSocketAddrs;
use std::time::Duration;
use mco::net::TcpStream;
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
    pub fn connector(self) -> RedisConnector<A> {
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
    fn _connect_timeout(&mut self, timeout: Duration) -> Result<SimpleClient, ConnectError> {
        let passwords = self.passwords.clone();
        let mut addrs = self.address.to_socket_addrs()?;
        let mut addr = None;
        loop {
            match addrs.next() {
                None => { break; }
                Some(v) => {
                    addr = Some(v);
                }
            }
        }
        if addr.is_none() {
            return Err(ConnectError::Connect("none socket addr!".to_string()));
        }
        let addr = addr.unwrap();
        let conn = TcpStream::connect_timeout(&addr, timeout.clone())?;
        conn.set_read_timeout(Some(timeout.clone()));
        conn.set_write_timeout(Some(timeout));
        if passwords.is_empty() {
            Ok(SimpleClient::new(conn))
        } else {
            let client = SimpleClient::new(conn);
            for password in passwords {
                if client.exec(cmd::Auth(password))? {
                    return Ok(client);
                }
            }
            Err(ConnectError::Unauthorized)
        }
    }

    fn _connect(&mut self) -> Result<SimpleClient, ConnectError> {
        let passwords = self.passwords.clone();
        let conn = TcpStream::connect(self.address.clone())?;
        if passwords.is_empty() {
            Ok(SimpleClient::new(conn))
        } else {
            let client = SimpleClient::new(conn);
            for password in passwords {
                if client.exec(cmd::Auth(password))? {
                    return Ok(client);
                }
            }
            Err(ConnectError::Unauthorized)
        }
    }

    /// Connect to redis server and create shared client
    pub fn connect(&mut self) -> Result<Client, ConnectError> {
        Ok(Client::new(self._connect()?))
    }

    /// Connect to redis server and create shared client with timeout
    pub fn connect_timeout(&mut self, timeout: Duration) -> Result<Client, ConnectError> {
        Ok(Client::new(self._connect_timeout(timeout)?))
    }

    /// Connect to redis server and create simple client
    pub fn connect_simple(&mut self) -> Result<SimpleClient, ConnectError> {
        Ok(self._connect()?)
    }

    /// Connect to redis server and create simple client
    pub fn connect_simple_timeout(&mut self, timeout: Duration) -> Result<SimpleClient, ConnectError> {
        Ok(self._connect_timeout(timeout)?)
    }
}
