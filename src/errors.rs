//! Redis protocol related errors
use std::io;

use derive_more::{Display, From};
use crate::bytes::ByteString;
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

    #[display(fmt = "Command error: {:?}", _0)]
    Command(String),

    #[display(fmt = "Recv error: {:?}", _0)]
    Recv(String),
}

impl std::error::Error for Error {}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Parse(s) => Error::Parse(s.clone()),
            Error::PeerGone(_) => Error::PeerGone(None),
            Error::Command(v)=> Error::Command(v.clone()),
            Error::Recv(v)=> Error::Recv(v.clone()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::PeerGone(Some(err))
    }
}

impl From<CommandError> for Error{
    fn from(arg: CommandError) -> Self {
        Error::Command(format!("{:?}",arg))
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

impl From<std::io::Error> for ConnectError {
    fn from(arg: std::io::Error) -> Self {
        ConnectError::Connect(arg.to_string())
    }
}

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

impl From<std::io::Error> for CommandError{
    fn from(arg: io::Error) -> Self {
        CommandError::Protocol(Error::PeerGone(Some(arg)))
    }
}