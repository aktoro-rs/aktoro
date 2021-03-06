use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;
use futures_core::FusedStream;
use futures_core::Stream;

use crate::channel::Channel;
use crate::error::*;

/// A channel's receiver allowing to
/// get messages from it either via the
/// [`try_recv`] method or the `Stream`
/// implementation.
///
/// [`try_recv`]: #method.try_recv
pub struct Receiver<T> {
    /// The channel the receiver will
    /// receive data from.
    channel: Option<Arc<Channel<T>>>,
    /// A reference to the space the
    /// receiver was assigned to store
    /// its waker.
    waker: Arc<AtomicCell<(bool, Option<Waker>)>>,
}

impl<T> Receiver<T> {
    /// Creates a receiver from a pointer
    /// to a channel.
    pub(crate) fn new(channel: Arc<Channel<T>>) -> Self {
        // This shouldn't fail because this
        // method should only be called
        // immediately after a channel's
        // creation.
        channel.counters.add_recver().expect("receivers limit == 0");

        let waker = Arc::new(AtomicCell::new((true, None)));
        channel.register(waker.clone());

        Receiver {
            waker,
            channel: Some(channel),
        }
    }

    /// Tries to receive a message from
    /// the channel.
    pub fn try_recv(&self) -> Result<Option<T>, TryRecvError> {
        if let Some(channel) = &self.channel {
            match channel.try_recv() {
                Ok(Some(msg)) => Ok(Some(msg.unwrap())),
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            }
        } else {
            Err(TryRecvError::disconnected())
        }
    }

    /// Whether the channel the receiver
    /// is connected to is closed.
    pub fn is_closed(&self) -> bool {
        if let Some(channel) = &self.channel {
            channel.check_is_closed()
        } else {
            true
        }
    }

    /// Closes the channel the receiver
    /// is connected to.
    pub fn close_channel(&self) {
        self.waker.store((false, None));

        if let Some(channel) = &self.channel {
            channel.close()
        }
    }

    /// Disconnects the receiver from the
    /// channel.
    pub fn disconnect(&mut self) {
        self.waker.store((false, None));

        let channel = if let Some(channel) = self.channel.take() {
            channel
        } else {
            return;
        };

        if channel.counters.sub_recver() == 0 {
            channel.close();
        }
    }

    /// Tries to clone the receiver, either
    /// returning a new receiver connected to
    /// the same channel, or an error.
    pub fn try_clone(&self) -> Result<Self, CloneError> {
        if let Some(channel) = &self.channel {
            if channel.counters.add_recver().is_ok() {
                let waker = Arc::new(AtomicCell::new((true, None)));

                Ok(Receiver {
                    waker,
                    channel: Some(channel.clone()),
                })
            } else {
                Err(CloneError::limit())
            }
        } else {
            Err(CloneError::disconnected())
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<T>> {
        self.waker.store((true, None));

        if let Some(channel) = &self.channel {
            // We try to receive a message...
            match channel.try_recv() {
                // ...and return it if one is
                // available...
                Ok(Some(msg)) => Poll::Ready(Some(msg.unwrap())),
                // ...or register the stream's
                // waker if none is available...
                Ok(None) => {
                    self.waker.store((true, Some(ctx.waker().clone())));

                    Poll::Pending
                }
                // ...or stop the stream if
                // an error occured.
                Err(_) => Poll::Ready(None),
            }
        } else {
            Poll::Ready(None)
        }
    }
}

impl<T> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        if let Some(channel) = &self.channel {
            channel.is_closed() && channel.is_empty()
        } else {
            true
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.disconnect();
    }
}
