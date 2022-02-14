use std::sync::Arc;
use cogo::go;
use either::Either;
use crate::codec_redis::{Codec, Request, Response};
use super::cmd::Command;
use super::errors::{CommandError, Error};
use cogo::std::sync::{Receiver, Sender, SyncQueue};
use crate::pool::Pool;
use crate::simple::SimpleClient;

#[derive(Clone)]
/// Shared redis client
pub struct Client {
    io: SimpleClient,
    queue: Arc<SyncQueue<Sender<Result<Response, Error>>>>,
    pool: Pool,
}

impl Client {
    pub(crate) fn new(io: SimpleClient) -> Self {
        let queue = Arc::new(SyncQueue::new());
        // read redis response task
        let io_ref = io.get_ref();
        let queue2 = queue.clone();
        go!(move ||{
            loop {
                match ready!(io.poll_recv(&Codec, cx)) {
                    Ok(item) => {
                        if let Some(tx) = queue2.borrow_mut().pop_front() {
                            let _ = tx.send(Ok(item));
                        } else {
                            log::error!("Unexpected redis response: {:?}", item);
                        }
                        continue;
                    }
                    Err(RecvError::KeepAlive) | Err(RecvError::Stop) => {
                        unreachable!()
                    }
                    Err(RecvError::WriteBackpressure) => {
                        if ready!(io.poll_flush(cx, false)).is_err() {
                            return Poll::Ready(());
                        } else {
                            continue;
                        }
                    }
                    Err(RecvError::Decoder(e)) => {
                        if let Some(tx) = queue2.borrow_mut().pop_front() {
                            let _ = tx.send(Err(e));
                        }
                        queue2.borrow_mut().clear();
                        let _ = ready!(io.poll_shutdown(cx));
                        return Poll::Ready(());
                    }
                    Err(RecvError::PeerGone(e)) => {
                        log::info!("Redis connection is dropped: {:?}", e);
                        queue2.borrow_mut().clear();
                        return Poll::Ready(());
                    }
                }
            }
        });
        Client {
            queue,
            io: io_ref,
            pool: Pool::new(),
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
                    v.map_err(CommandError::Protocol)
                }
                Either::Right(v) => {
                    T::to_output(v.map_err(CommandError::Error)?)
                }
            }
        }
    }

    /// Delete all the keys of the currently selected DB.
    pub async fn flushdb(&self) -> Result<(), Error> {
        self.call("FLUSHDB".into()).await?;
        Ok(())
    }

    /// Returns true if underlying transport is connected to redis
    pub fn is_connected(&self) -> bool {
        !self.io.is_closed()
    }
}

impl Client {
    fn call(&self, req: Request) -> Either<CommandResult, Result<Response, Error>> {
        if let Err(e) = self.io.encode(req, &Codec) {
            Either::Right(Err(e))
        } else {
            let (tx, rx) = self.pool.channel();
            self.queue.push(tx);
            match rx.recv() {
                Ok(v) => {
                    Either::Left(CommandResult::Ok(v))
                }
                Err(e) => {
                    Either::Left(CommandResult::Err(e))
                }
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

pub type CommandResult = Result<Response, Error>;