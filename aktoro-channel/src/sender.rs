use std::sync::Arc;

use crate::channel::Channel;
use crate::error::*;
use crate::message::Message;
use crate::notify::Notify;

pub struct Sender<T>(Option<Arc<Channel<T>>>);

impl<T> Sender<T> {
    pub(crate) fn new(channel: Arc<Channel<T>>) -> Self {
        channel.counters.add_sender().expect("senders limit == 0");
        Sender(Some(channel))
    }

    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        if let Some(channel) = &self.0 {
            channel.try_send(Message::new(msg))
        } else {
            Err(TrySendError::disconnected(msg))
        }
    }

    pub fn try_send_notify(&self, msg: T) -> Result<Notify, TrySendError<T>> {
        if let Some(channel) = &self.0 {
            let (msg, notify) = Message::new_notified(msg);

            channel.try_send(msg)?;
            Ok(notify)
        } else {
            Err(TrySendError::disconnected(msg))
        }
    }

    pub fn is_closed(&self) -> bool {
        if let Some(channel) = &self.0 {
            channel.is_closed()
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

        channel.counters.sub_sender();
    }

    pub fn try_clone(&self) -> Result<Self, CloneError> {
        if let Some(channel) = &self.0 {
            if channel.counters.add_sender().is_ok() {
                Ok(Sender(Some(channel.clone())))
            } else {
                Err(CloneError::limit())
            }
        } else {
            Err(CloneError::disconnected())
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.disconnect();
    }
}
