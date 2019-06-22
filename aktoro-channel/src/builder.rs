use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use crossbeam_queue::SegQueue;

use crate::channel::Channel;
use crate::counters::Counters;
use crate::queue::Queue;
use crate::receiver::Receiver;
use crate::sender::Sender;

/// A configuration builder for a
/// channel.
pub struct Builder {
    /// The capacity of the future
    /// channel, or `None` if it
    /// should be unbounded.
    cap: Option<usize>,
    /// The limit of messages that
    /// can be send over the channel,
    /// or `None` if no limit should
    /// be set.
    msgs: Option<usize>,
    /// The limit of senders the
    /// channel should be allowed to
    /// have at the same time, or
    /// `None` if no limit should
    /// be set.
    senders: Option<usize>,
    /// The limit of receivers the
    /// channel should be allowed to
    /// have at the same time, or
    /// `None` if no limit should
    /// be set.
    recvers: Option<usize>,
}

impl Builder {
    /// Creates a new builder with
    /// the default configuration:
    /// - unbounded
    /// - no limit of messages
    /// - no limit of senders
    /// - no limit of receivers
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Sets the channel to be created
    /// as bounded with `cap` as its
    /// buffer capacity.
    pub fn bounded(mut self, cap: usize) -> Builder {
        self.cap = Some(cap);
        self
    }

    /// Sets the channel to be created
    /// as unbounded.
    pub fn unbounded(mut self) -> Builder {
        self.cap = None;
        self
    }

    /// Sets the maximum number of
    /// messages that the channel will
    /// be able to pass.
    pub fn limited_msgs(mut self, limit: usize) -> Builder {
        self.msgs = Some(limit);
        self
    }

    /// Allows an infinite number of
    /// messages to be sent over the
    /// channel.
    pub fn unlimited_msgs(mut self) -> Builder {
        self.msgs = None;
        self
    }

    /// Sets the maximum number of
    /// senders that the channel will
    /// be able to have at the same
    /// time.
    pub fn limited_senders(mut self, limit: usize) -> Builder {
        self.senders = Some(limit);
        self
    }

    /// Alows an inifinite number of
    /// senders to be connected to
    /// the channel at the same time.
    pub fn unlimited_senders(mut self) -> Builder {
        self.senders = None;
        self
    }

    /// Sets the maximum number of
    /// receivers that will be allowed
    /// to be connected to the channel
    /// at the same time.
    pub fn limited_receivers(mut self, limit: usize) -> Builder {
        self.recvers = Some(limit);
        self
    }

    /// Allows an inifinite number of
    /// receivers to be connected
    /// to the channel at the same time.
    pub fn unlimited_receivers(mut self) -> Builder {
        self.recvers = None;
        self
    }

    /// Builds the channel using the
    /// specified configuration and
    /// returning a sender and receiver
    /// connected to it.
    pub fn build<T>(self) -> (Sender<T>, Receiver<T>) {
        // We create either a bounded or
        // unbounded queue.
        let queue = if let Some(cap) = self.cap {
            Queue::Bounded(ArrayQueue::new(cap))
        } else {
            Queue::Unbounded(SegQueue::new())
        };

        // We create the channel and put
        // it inside an atomically
        // reference counted pointer.
        let channel = Arc::new(Channel {
            queue,
            closed: AtomicBool::new(false),
            counters: Counters::new(self.msgs, self.senders, self.recvers),
            wakers: SegQueue::new(),
        });

        // We return a sender and a
        // receiver containing a copy
        // of the pointer.
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
