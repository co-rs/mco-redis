use cogo::chan;
use cogo::std::sync::{Receiver, Sender, SyncQueue};
use crate::codec_redis::Response;
use crate::errors::Error;

pub struct Pool {
    // pub clients: SyncQueue<SimpleClient>,
    pub chans: SyncQueue<(Sender<Result<Response, Error>>, Receiver<Result<Response, Error>>)>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            chans: SyncQueue::new()
        }
    }

    pub fn channel(&self) {
        let (s, r) = chan!();
        self.chans.push((s.clone(), r.clone()));
    }
}