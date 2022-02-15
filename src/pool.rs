use std::sync::Arc;
use cogo::chan;
use cogo::std::sync::{Receiver, Sender, SyncQueue};
use crate::codec_redis::Response;
use crate::errors::Error;

#[derive(Clone)]
pub struct Pool {
    // pub clients: SyncQueue<SimpleClient>,
    pub chans: Arc<SyncQueue<(Sender<Result<Response, Error>>, Receiver<Result<Response, Error>>)>>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            chans: Arc::new(SyncQueue::new())
        }
    }

    pub fn channel(&self) {
        let (s, r) = chan!();
        self.chans.push((s.clone(), r.clone()));
    }
}