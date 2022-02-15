use std::sync::Arc;
use std::sync::mpsc::RecvError;
use cogo::chan;
use cogo::coroutine::spawn;
use either::Either;
use crate::codec_redis::{Codec, Request, Response};
use super::cmd::Command;
use super::errors::{CommandError, Error};
use cogo::std::sync::{Receiver, Sender};
use crate::bytes::BytesMut;
use crate::simple::SimpleClient;

pub type CommandResult = Result<Response, Error>;

/// Shared redis client
#[derive(Clone)]
pub struct Client {
    io: Arc<SimpleClient>,
    queue: Arc<(Sender<(BytesMut, Sender<Result<Response, Error>>)>, Receiver<(BytesMut, Sender<Result<Response, Error>>)>)>,
}

impl Client {
    pub(crate) fn new(io: SimpleClient) -> Self {
        let io = Arc::new(io);
        let queue = Arc::new(chan!());
        // read redis response task
        let io_clone= io.clone();
        let queue_clone = queue.clone();
        spawn(move || {
            loop {
                match queue_clone.1.recv() {
                    Ok((data,s)) => {
                        let s:Sender<Result<Response, Error>> = s;
                        s.send(match io_clone.send(&data){
                            Ok(d)=>{
                                Ok(d)
                            }
                            Err(e)=>{
                                Err(e.into())
                            }
                        });
                    }
                    Err(e) => {
                        log::info!("Redis client is dropped: {:?}", e);
                        break;
                    }
                }
            }
        });
        Client {
            queue: queue,
            io: io,
        }
    }

    /// Execute redis command
    pub fn exec<T>(&self, cmd: T) -> Result<T::Output, CommandError>
        where
            T: Command,
    {
        let is_open = !self.io.is_closed();
        let result = self.call(cmd.to_request());
        if !is_open {
            Err(CommandError::Protocol(Error::PeerGone(None)))
        } else {
            match result {
                Either::Left(v) => {
                    match v{
                        Ok(res) => {
                            T::to_output(res.into_result().map_err(CommandError::Error)?)
                        }
                        Err(e) => {
                          Err(CommandError::Protocol(e))
                        }
                    }
                }
                Either::Right(v) => {
                    v.map_err(CommandError::Protocol)
                        .and_then(|res| T::to_output(res.into_result().map_err(CommandError::Error)?))
                }
            }
        }
    }

    /// Delete all the keys of the currently selected DB.
    pub fn flushdb(&self) -> Result<(), Error> {
        match self.call("FLUSHDB".into()){
            Either::Left(v) => {
                v?;
            }
            Either::Right(v) => {
                v?;
            }
        }
        Ok(())
    }

    /// Returns true if underlying transport is connected to redis
    pub fn is_connected(&self) -> bool {
        !self.io.is_closed()
    }

    /// call and return Either
    pub fn call(&self, req: Request) -> Either<CommandResult, Result<Response, Error>> {
        let mut buf = BytesMut::new();
        match self.io.encode_req(req,&mut buf) {
            Ok(_) => {
                let (tx, rx) = chan!();
                self.queue.0.send((buf,tx));
                match rx.recv() {
                    Ok(v) => {
                        match v{
                            Ok(v) => {
                                Either::Left(CommandResult::Ok(v))
                            }
                            Err(e) => {
                                Either::Left(CommandResult::Err(e))
                            }
                        }
                    }
                    Err(e) => {
                        Either::Right(Err(Error::Recv(e.to_string())))
                    }
                }
            }
            Err(e) => {
                Either::Right(Err(e))
            }
        }
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("connected", &!self.io.is_closed())
            .finish()
    }
}