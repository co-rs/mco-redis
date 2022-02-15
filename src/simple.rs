use std::cell::RefCell;
use std::io;
use std::io::{Read, Write};
use cogo::net::TcpStream;
use crate::bytes::{BufMut, BytesMut, ByteString};
use crate::codec::{Decoder, Encoder};
use crate::codec_redis::{Codec, Request, Response};
use crate::errors::Error;

use super::cmd::Command;
use super::errors::{CommandError};

/// Redis client
pub struct SimpleClient {
    pub codec: Codec,
    pub io: RefCell<Option<TcpStream>>,
}

unsafe impl Send for SimpleClient {}

unsafe impl Sync for SimpleClient {}

impl SimpleClient {
    /// Create new simple client
    pub fn new(io: TcpStream) -> Self {
        SimpleClient { codec: Codec {}, io: RefCell::new(Some(io)) }
    }

    /// Execute redis command
    pub fn exec<U>(&self, cmd: U) -> Result<U::Output, CommandError>
        where
            U: Command,
    {
        let buf = self.encode(cmd)?;
        let resp = self.send(&buf)?;
        self.decode::<U>(resp)
    }

    pub fn encode<U: Command>(&self, cmd: U) -> Result<BytesMut, Error> {
        let mut buf_in = BytesMut::new();
        let mut req = cmd.to_request();
        self.codec.encode(req, &mut buf_in)?;
        Ok(buf_in)
    }

    pub fn encode_req(&self, req: Request, buf: &mut BytesMut) -> Result<(), Error> {
        self.codec.encode(req, buf)?;
        Ok(())
    }

    pub fn send(&self, arg: &BytesMut) -> Result<Response, CommandError> {
        let mut io = self.io.borrow_mut();
        if io.is_none() {
            return Err(CommandError::Protocol(Error::PeerGone(None)));
        }
        let io = io.as_mut().unwrap();
        io.write_all(arg)?;
        io.flush();
        let mut buffer = BytesMut::with_capacity(64);
        loop {
            let mut buf = BytesMut::with_capacity(64);
            buf.put(&[0; 64][..]);
            io.read(&mut buf)?;
            buffer.extend(buf);
            match self.codec.decode(&mut buffer)? {
                None => {
                    continue;
                }
                Some(item) => {
                    return Ok(item);
                }
            }
        }
    }

    pub fn decode<U>(&self, resp: Response) -> Result<U::Output, CommandError>
        where
            U: Command, {
        return U::to_output(resp.into_result().map_err(CommandError::Error)?);
    }

    pub fn is_closed(&self) -> bool {
        self.io.borrow().is_none()
    }
}




