use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures_core::stream::FusedStream;
use futures_core::stream::Stream;
use futures_sink::Sink;

use crate::bounded;
use crate::error::*;
use crate::unbounded;

/// Creates a new bounded channel (see [`futures-channel`'s
/// documentation]).
///
/// [`futures-channel`'s documentation]: https://docs.rs/futures-channel-preview/0.3.0-alpha.16/futures_channel/mpsc/fn.channel.html
pub fn bounded<D>(buf: usize) -> (Sender<D>, Receiver<D>) {
    let (sender, receiver) = bounded::new(buf);

    (Sender::Bounded(sender), Receiver::Bounded(receiver))
}

/// Creates a new unbounded channel (see [`futures-channel`'s
/// documentation]).
///
/// [`futures-channel`'s documentation]: https://docs.rs/futures-channel-preview/0.3.0-alpha.16/futures_channel/mpsc/fn.unbounded.html
pub fn unbounded<D>() -> (Sender<D>, Receiver<D>) {
    let (sender, receiver) = unbounded::new();

    (Sender::Unbounded(sender), Receiver::Unbounded(receiver))
}

#[derive(Debug)]
/// A wrapper around either a [`bounded::Sender`] or a
/// [`unbounded::Sender`] that allows to use all the
/// methods and traits that they implement.
///
/// [`bounded::Sender`]: bounded/struct.Sender.html
/// [`unbounded::Sender`]: unbounded/struct.Sender.html
pub enum Sender<D> {
    Bounded(bounded::Sender<D>),
    Unbounded(unbounded::Sender<D>),
}

#[derive(Debug)]
/// A wrapper around either a [`bounded::Receiver`] or
/// a [`unbounded::Receiver`] that allows to use all
/// the methods and trait that they implement.
///
/// [`bounded::Receiver`]: bounded/struct.Receiver.html
/// [`unbounded::Receiver`]: unbounded/struct.Receiver.html
pub enum Receiver<D> {
    Bounded(bounded::Receiver<D>),
    Unbounded(unbounded::Receiver<D>),
}

impl<D> Sender<D> {
    /// Sends `data` over the channel, returning `Ok(())` if
    /// it has been successfully sent, or either
    /// `Err(SendError::Full)` if the channel's buffer is full,
    /// `Err(SendError::Disconnected)` if the  sender has
    /// disconnected itself from the channel or
    /// `Err(SendError::Closed)` if the channel has been closed.
    pub fn send(&mut self, data: D) -> Result<(), SendError<D>> {
        match self {
            Sender::Bounded(sender) => sender.send(data),
            Sender::Unbounded(sender) => sender.send(data),
        }
    }

    /// Tries to disconnect the sender from the channel,
    /// returning `Ok(())` if it succeeded, or either
    /// `Err(DiconnectError::Disconnected)` if the sender
    /// already disconnected itself, or
    /// `Err(DisconnectError::Closed)` if the channel was
    /// already closed.
    pub fn disconnect(&mut self) -> Result<(), DisconnectError> {
        match self {
            Sender::Bounded(sender) => sender.disconnect(),
            Sender::Unbounded(sender) => sender.disconnect(),
        }
    }

    /// Tries to close the channel, returning `Ok(())` if it
    /// succeeded, or either `Err(CloseError::Disconnected)`
    /// if the sender already disconnected itself, or
    /// `Err(CloseError::Closed)` if the channel was already
    /// closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        match self {
            Sender::Bounded(sender) => sender.close(),
            Sender::Unbounded(sender) => sender.close(),
        }
    }
}

impl<D> Receiver<D> {
    /// Tries to receive a message over the channel, returning
    /// `Ok(D)` if it has received one, and either
    /// `Err(ReceiveError::Empty)` if it hasn't or
    /// `Err(ReceiveError::Closed)` if the channel has been
    /// closed.
    pub fn try_recv(&mut self) -> Result<D, ReceiveError> {
        match self {
            Receiver::Bounded(receiver) => receiver.try_recv(),
            Receiver::Unbounded(receiver) => receiver.try_recv(),
        }
    }

    /// Tries to close the channel, returning `Ok(())` if it
    /// succeeded, or `Err(CloseError::Closed)` if the channel
    /// was already closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        match self {
            Receiver::Bounded(receiver) => receiver.close(),
            Receiver::Unbounded(receiver) => receiver.close(),
        }
    }
}

impl<D> Unpin for Receiver<D> {}

impl<D> Sink<D> for Sender<D> {
    // FIXME: -`()` +`D` (the issue being that `poll_ready`,
    //   `poll_flush` and `poll_close` can't return `D` since
    //   they don't get any data in the first place).
    type SinkError = SendError<()>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        match self.get_mut() {
            Sender::Bounded(sender) => Pin::new(sender).poll_ready(cx),
            Sender::Unbounded(sender) => Pin::new(sender).poll_ready(cx),
        }
    }

    fn start_send(self: Pin<&mut Self>, msg: D) -> Result<(), SendError<()>> {
        match self.get_mut() {
            Sender::Bounded(sender) => Pin::new(sender).start_send(msg),
            Sender::Unbounded(sender) => Pin::new(sender).start_send(msg),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        match self.get_mut() {
            Sender::Bounded(sender) => Pin::new(sender).poll_flush(cx),
            Sender::Unbounded(sender) => Pin::new(sender).poll_flush(cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        match self.get_mut() {
            Sender::Bounded(sender) => Pin::new(sender).poll_close(cx),
            Sender::Unbounded(sender) => Pin::new(sender).poll_close(cx),
        }
    }
}

impl<D> FusedStream for Receiver<D> {
    fn is_terminated(&self) -> bool {
        match self {
            Receiver::Bounded(receiver) => receiver.is_terminated(),
            Receiver::Unbounded(receiver) => receiver.is_terminated(),
        }
    }
}

impl<D> Stream for Receiver<D> {
    type Item = D;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<D>> {
        match self.get_mut() {
            Receiver::Bounded(receiver) => Pin::new(receiver).poll_next(cx),
            Receiver::Unbounded(receiver) => Pin::new(receiver).poll_next(cx),
        }
    }
}

impl<D> Clone for Sender<D> {
    fn clone(&self) -> Sender<D> {
        match self {
            Sender::Bounded(sender) => Sender::Bounded(sender.clone()),
            Sender::Unbounded(sender) => Sender::Unbounded(sender.clone()),
        }
    }
}
