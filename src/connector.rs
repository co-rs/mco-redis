use std::future::Future;
use crate::bytes::{ByteString, PoolId, PoolRef};
use crate::client::Client;
use crate::cmd;
use crate::simple::SimpleClient;
use super::errors::ConnectError;

/// Redis connector
pub struct RedisConnector<A, T> {
    address: A,
    connector: T,
    passwords: Vec<ByteString>,
    pool: PoolRef,
}

impl<A> RedisConnector<A, ()>
where
    A: Address + Clone,
{
    #[allow(clippy::new_ret_no_self)]
    /// Create new redis connector
    pub fn new(address: A) -> RedisConnector<A, Connector<A>> {
        RedisConnector {
            address,
            passwords: Vec::new(),
            connector: Connector::default(),
            pool: PoolId::P7.pool_ref(),
        }
    }
}

impl<A, T> RedisConnector<A, T>
where
    A: Address + Clone,
{
    /// Add redis auth password
    pub fn password<U>(mut self, password: U) -> Self
    where
        U: AsRef<str>,
    {
        self.passwords
            .push(ByteString::from(password.as_ref().to_string()));
        self
    }

    /// Set memory pool.
    ///
    /// Use specified memory pool for memory allocations. By default P7
    /// memory pool is used.
    pub fn memory_pool(mut self, id: PoolId) -> Self {
        self.pool = id.pool_ref();
        self
    }

    /// Use custom connector
    pub fn connector<U>(self, connector: U) -> RedisConnector<A, U>
    where
        U: Service<Connect<A>, Error = connect::ConnectError>,
        IoBoxed: From<U::Response>,
    {
        RedisConnector {
            connector,
            address: self.address,
            passwords: self.passwords,
            pool: self.pool,
        }
    }
}

impl<A, T> RedisConnector<A, T>
where
    A: Address + Clone,
    T: Service<Connect<A>, Error = connect::ConnectError>,
    IoBoxed: From<T::Response>,
{
    fn _connect(&self) -> Result<IoBoxed, ConnectError> {
        let pool = self.pool;
        let passwords = self.passwords.clone();
        let fut = self.connector.call(Connect::new(self.address.clone()));
        let io = IoBoxed::from(fut.await?);
        io.set_memory_pool(pool);
        io.set_disconnect_timeout(Seconds::ZERO.into());

        if passwords.is_empty() {
            Ok(io)
        } else {
            let client = SimpleClient::new(io);

            for password in passwords {
                if client.exec(cmd::Auth(password)).await? {
                    return Ok(client.into_inner());
                }
            }
            Err(ConnectError::Unauthorized)
        }
    }

    /// Connect to redis server and create shared client
    pub fn connect(&self) -> Result<Client, ConnectError> {
        Ok(Client::new(self._connect()?))
    }

    /// Connect to redis server and create simple client
    pub fn connect_simple(&self) -> Result<SimpleClient, ConnectError> {
        Ok(SimpleClient::new(self._connect()?))
    }
}