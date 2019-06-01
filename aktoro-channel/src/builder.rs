use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use crossbeam_queue::SegQueue;

use crate::channel::Channel;
use crate::counters::Counters;
use crate::queue::Queue;
use crate::receiver::Receiver;
use crate::sender::Sender;

pub struct Builder {
    cap: Option<usize>,
    msgs: Option<usize>,
    senders: Option<usize>,
    recvers: Option<usize>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn bounded(mut self, cap: usize) -> Builder {
        self.cap = Some(cap);
        self
    }

    pub fn unbounded(mut self) -> Builder {
        self.cap = None;
        self
    }

    pub fn limited_msgs(mut self, limit: usize) -> Builder {
        self.msgs = Some(limit);
        self
    }

    pub fn unlimited_msgs(mut self) -> Builder {
        self.msgs = None;
        self
    }

    pub fn limited_senders(mut self, limit: usize) -> Builder {
        self.senders = Some(limit);
        self
    }

    pub fn unlimited_senders(mut self) -> Builder {
        self.senders = None;
        self
    }

    pub fn limited_receivers(mut self, limit: usize) -> Builder {
        self.recvers = Some(limit);
        self
    }

    pub fn unlimited_receivers(mut self) -> Builder {
        self.recvers = None;
        self
    }

    pub fn build<T>(self) -> (Sender<T>, Receiver<T>) {
        let queue = if let Some(cap) = self.cap {
            Queue::Bounded(ArrayQueue::new(cap))
        } else {
            Queue::Unbounded(SegQueue::new())
        };

        let channel = Arc::new(Channel {
            queue,
            closed: AtomicBool::new(false),
            counters: Counters::new(self.msgs, self.senders, self.recvers),
            wakers: SegQueue::new(),
        });

        (Sender::new(channel.clone()), Receiver::new(channel))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            cap: None,
            msgs: None,
            senders: None,
            recvers: None,
        }
    }
}
