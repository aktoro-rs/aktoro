use std::sync::Arc;

use crate::channel::Channel;
use crate::error::*;
use crate::message::Message;
use crate::notify::Notify;

/// A channel's sender allowing to
/// send messages over it.
pub struct Sender<T>(Option<Arc<Channel<T>>>);

impl<T> Sender<T> {
    /// Creates a new sender from a pointer
    /// to a channel.
    pub(crate) fn new(channel: Arc<Channel<T>>) -> Self {
        // This shouldn't fail because this
        // method should only be called
        // immediately after a channel's
        // creation.
        channel.counters.add_sender().expect("senders limit == 0");
        Sender(Some(channel))
    }

    /// Tries to send a message over the
    /// channel, returning an error if the
    /// channel is full or the maximum
    /// number of messages that can be sent
    /// over it has been reached.
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        if let Some(channel) = &self.0 {
            channel.try_send(Message::new(msg))
        } else {
            Err(TrySendError::disconnected(msg))
        }
    }

    /// Tries to send a message over the
    /// channel, returning an error if the
    /// channel is full or the maximum
    /// number of messages that can be sent
    /// over it has been reached; or a
    /// [`Notify`] future that will resolve
    /// when the message has been received
    /// and handled.
    ///
    /// [`Notify`]: struct.Notify.html
    pub fn try_send_notify(&self, msg: T) -> Result<Notify, TrySendError<T>> {
        if let Some(channel) = &self.0 {
            let (msg, notify) = Message::new_notified(msg);

            channel.try_send(msg)?;
            Ok(notify)
        } else {
            Err(TrySendError::disconnected(msg))
        }
    }

    /// Whether the channel the sender is
    /// connected to is closed.
    pub fn is_closed(&self) -> bool {
        if let Some(channel) = &self.0 {
            channel.is_closed()
        } else {
            true
        }
    }

    /// Closes the channel the sender is
    /// connected to.
    pub fn close_channel(&self) {
        if let Some(channel) = &self.0 {
            channel.close()
        }
    }

    /// Disconnects the sender from the
    /// channel it is connected to.
    pub fn disconnect(&mut self) {
        let channel = if let Some(channel) = self.0.take() {
            channel
        } else {
            return;
        };

        channel.counters.sub_sender();
    }

    /// Tries to clone the sender, either
    /// returning a new sender connected to
    /// the same channel, or an error.
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
