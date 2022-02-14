use std::cell::RefCell;
use std::io::{Read, Write};
use std::task::Poll;
use cogo::net::TcpStream;
use crate::bytes::BytesMut;
use crate::codec::{Decoder, EncoderDecoder};
use crate::codec_redis::{Codec, Request};

use super::cmd::Command;
use super::errors::{CommandError, Error};

/// Redis client
pub struct SimpleClient {
    pub codec: Box<dyn EncoderDecoder>,
    pub io: RefCell<TcpStream>,
}

impl SimpleClient {
    /// Create new simple client
    pub fn new(io: TcpStream) -> Self {
        SimpleClient { codec: Box::new(Codec {}), io: RefCell::new(io) }
    }

    /// Execute redis command
    pub fn exec<U>(&self, cmd: U) -> Result<U::Output, CommandError>
        where
            U: Command,
    {
        let mut buf_in = BytesMut::new();
        let mut req = cmd.to_request();
        self.codec.encode(req, &mut buf_in)?;
        self.io.borrow_mut().write_all(&buf_in)?;
        self.io.borrow_mut().flush();

        let mut buf_out = BytesMut::new();
        self.io.borrow_mut().read(&mut buf_out)?;

        loop {
            match self.codec.decode(&mut buf_out)? {
                None => {
                    continue;
                }
                Some(item) => {
                    return U::to_output(item.into_result().map_err(CommandError::Error)?);
                }
            }
        }


        // poll_fn(|cx| loop {
        //     return match ready!(self.io.poll_recv(&Codec, cx)) {
        //         Ok(item) => Poll::Ready(U::to_output(
        //             item.into_result().map_err(CommandError::Error)?,
        //         )),
        //         Err(RecvError::KeepAlive) | Err(RecvError::Stop) => {
        //             unreachable!()
        //         }
        //         Err(RecvError::WriteBackpressure) => {
        //             ready!(self.io.poll_flush(cx, false))
        //                 .map_err(|e| CommandError::Protocol(Error::PeerGone(Some(e))))?;
        //             continue;
        //         }
        //         Err(RecvError::Decoder(err)) => Poll::Ready(Err(CommandError::Protocol(err))),
        //         Err(RecvError::PeerGone(err)) => {
        //             Poll::Ready(Err(CommandError::Protocol(Error::PeerGone(err))))
        //         }
        //     };
        // })
        //     .await
    }
}
