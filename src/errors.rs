//! Redis protocol related errors
use std::io;

use derive_more::{Display, From};
use crate::cogo_bytes::ByteString;

use super::codec_redis::Response;

#[derive(Debug, Display)]
/// Redis protocol errors
pub enum Error {
    /// A RESP parsing error occurred
    #[display(fmt = "Redis server response error: {}", _0)]
    Parse(String),

    /// An IO error occurred
    #[display(fmt = "Io error: {:?}", _0)]
    PeerGone(Option<io::Error>),
}

impl std::error::Error for Error {}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Parse(_) => Error::Parse(String::new()),
            Error::PeerGone(_) => Error::PeerGone(None),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::PeerGone(Some(err))
    }
}

#[derive(Debug, Display, From)]
/// Redis connectivity errors
pub enum ConnectError {
    /// Auth command failed
    Unauthorized,
    /// Command execution error
    Command(CommandError),
    /// Io connectivity error
    Connect(String),
}

impl std::error::Error for ConnectError {}

#[derive(Debug, Display, From)]
/// Redis command execution errors
pub enum CommandError {
    /// A redis server error response
    Error(ByteString),

    /// A command response parse error
    #[display(fmt = "Command output parse error: {}", _0)]
    Output(&'static str, Response),

    /// Redis protocol level errors
    Protocol(Error),
}

impl std::error::Error for CommandError {}