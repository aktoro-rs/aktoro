use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use futures_core::FusedStream;
use futures_core::Stream;

use crate::channel::Channel;
use crate::error::*;

pub struct Receiver<T>(Option<Arc<Channel<T>>>);

impl<T> Receiver<T> {
    pub(crate) fn new(channel: Arc<Channel<T>>) -> Self {
        channel.counters.add_recver().expect("receivers limit == 0");
        Receiver(Some(channel))
    }

    pub fn try_recv(&self) -> Result<Option<T>, TryRecvError> {
        if let Some(channel) = &self.0 {
            match channel.try_recv() {
                Ok(Some(msg)) => Ok(Some(msg.unwrap())),
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            }
        } else {
            Err(TryRecvError::disconnected())
        }
    }

    pub fn is_closed(&self) -> bool {
        if let Some(channel) = &self.0 {
            channel.check_is_closed()
        } else {
            true
        }
    }

    pub fn close_channel(&self) {
        if let Some(channel) = &self.0 {
            channel.close()
        }
    }

    pub fn disconnect(&mut self) {
        let channel = if let Some(channel) = self.0.take() {
            channel
        } else {
            return;
        };

        if channel.counters.sub_recver() == 0 {
            channel.close();
        }
    }

    pub fn try_clone(&self) -> Result<Self, CloneError> {
        if let Some(channel) = &self.0 {
            if channel.counters.add_recver().is_ok() {
                Ok(Receiver(Some(channel.clone())))
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
        if let Some(channel) = &self.0 {
            match channel.try_recv() {
                Ok(Some(msg)) => Poll::Ready(Some(msg.unwrap())),
                Ok(None) => {
                    channel.register(ctx.waker().clone());
                    Poll::Pending
                }
                Err(_) => Poll::Ready(None),
            }
        } else {
            Poll::Ready(None)
        }
    }
}

impl<T> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        if let Some(channel) = &self.0 {
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
