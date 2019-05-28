use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures_channel::mpsc;
use futures_channel::mpsc::Receiver as FutReceiver;
use futures_channel::mpsc::Sender as FutSender;
use futures_core::stream::FusedStream;
use futures_core::stream::Stream;
use futures_sink::Sink;

use crate::error::*;

/// Creates a new bounded channel (see [`futures-channel`'s
/// documentation]).
///
/// [`futures-channel`'s documentation]: https://docs.rs/futures-channel-preview/0.3.0-alpha.16/futures_channel/mpsc/fn.channel.html
pub fn new<D>(buf: usize) -> (Sender<D>, Receiver<D>) {
    let (sender, receiver) = mpsc::channel(buf);

    (Sender::new(buf, sender), Receiver::new(buf, receiver))
}

#[derive(Debug)]
/// A wrapper around a [`mpsc::Sender`] that stores its state
/// after sending data, closing the channel or disconnecting
/// itself.
///
/// [`mpsc::Sender`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures/channel/mpsc/struct.Sender.html
pub struct Sender<D> {
    /// The size of the buffer (as it was provided to
    /// [`bounded`])
    ///
    /// [`bounded`]: ../fn.bounded.html
    pub buf: usize,
    /// Whether the channel has been closed.
    pub closed: bool,
    /// Whether the sender has disconnected itself from the
    /// channel.
    pub disconnected: bool,
    sender: FutSender<D>,
}

#[derive(Debug)]
/// A wrapper around a [`mpsc::Receiver`] that stores its
/// state after trying to receive data or closing the channel.
///
/// [`mpsc::Receiver`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures/channel/mpsc/struct.Receiver.html
pub struct Receiver<D> {
    /// The size of the buffer (as it was provided to
    /// [`bounded`])
    ///
    /// [`bounded`]: ../fn.bounded.html
    pub buf: usize,
    /// Whether the channel has been closed.
    pub closed: bool,
    receiver: Option<FutReceiver<D>>,
}

impl<D> Sender<D> {
    pub(crate) fn new(buf: usize, sender: FutSender<D>) -> Sender<D> {
        Sender {
            buf,
            closed: false,
            disconnected: false,
            sender,
        }
    }

    /// Sends `data` over the channel, returning `Ok(())` if
    /// it has been successfully sent, or either
    /// `Err(SendError::Full)` if the channel's buffer is full,
    /// `Err(SendError::Disconnected)` if the  sender has
    /// disconnected itself from the channel or
    /// `Err(SendError::Closed)` if the channel has been closed.
    pub fn send(&mut self, data: D) -> Result<(), SendError<D>> {
        if self.disconnected {
            return Err(SendError::Disconnected(data));
        } else if self.closed {
            return Err(SendError::Closed(data));
        } else if self.sender.is_closed() {
            self.closed = true;
            return Err(SendError::Closed(data));
        }

        match self.sender.try_send(data) {
            Ok(()) => Ok(()),
            Err(err) => {
                if err.is_disconnected() {
                    self.closed = true;
                    Err(SendError::Closed(err.into_inner()))
                } else if err.is_full() {
                    Err(SendError::Full(err.into_inner()))
                } else {
                    unreachable!();
                }
            }
        }
    }

    /// Tries to disconnect the sender from the channel,
    /// returning `Ok(())` if it succeeded, or either
    /// `Err(DiconnectError::Disconnected)` if the sender
    /// already disconnected itself, or
    /// `Err(DisconnectError::Closed)` if the channel was
    /// already closed.
    pub fn disconnect(&mut self) -> Result<(), DisconnectError> {
        if self.disconnected {
            Err(DisconnectError::Disconnected)
        } else if self.closed {
            Err(DisconnectError::Closed)
        } else if self.sender.is_closed() {
            self.closed = true;
            Err(DisconnectError::Closed)
        } else {
            self.sender.disconnect();
            self.disconnected = true;
            Ok(())
        }
    }

    /// Tries to close the channel, returning `Ok(())` if it
    /// succeeded, or either `Err(CloseError::Disconnected)`
    /// if the sender already disconnected itself, or
    /// `Err(CloseError::Closed)` if the channel was already
    /// closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        if self.disconnected {
            Err(CloseError::Disconnected)
        } else if self.closed {
            Err(CloseError::Closed)
        } else if self.sender.is_closed() {
            self.closed = true;
            Err(CloseError::Closed)
        } else {
            self.sender.close_channel();
            self.closed = true;
            Ok(())
        }
    }
}

impl<D> Receiver<D> {
    pub(crate) fn new(buf: usize, receiver: FutReceiver<D>) -> Receiver<D> {
        Receiver {
            buf,
            closed: false,
            receiver: Some(receiver),
        }
    }

    /// Tries to receive a message over the channel, returning
    /// `Ok(D)` if it has received one, and either
    /// `Err(ReceiveError::Empty)` if it hasn't or
    /// `Err(ReceiveError::Closed)` if the channel has been
    /// closed.
    pub fn try_recv(&mut self) -> Result<D, ReceiveError> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.try_next() {
                Ok(Some(data)) => Ok(data),
                Ok(None) => {
                    self.receiver = None;
                    self.closed = true;
                    Err(ReceiveError::Closed)
                }
                Err(_) => Err(ReceiveError::Empty),
            }
        } else {
            Err(ReceiveError::Closed)
        }
    }

    /// Tries to close the channel, returning `Ok(())` if it
    /// succeeded, or `Err(CloseError::Closed)` if the channel
    /// was already closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        if self.closed {
            Err(CloseError::Closed)
        } else if let Some(ref mut receiver) = self.receiver {
            receiver.close();
            self.closed = true;
            Ok(())
        } else {
            unreachable!();
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
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).poll_ready(cx).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }

    fn start_send(self: Pin<&mut Self>, msg: D) -> Result<(), SendError<()>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).start_send(msg).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).poll_flush(cx).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).poll_close(cx).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }
}

impl<D> FusedStream for Receiver<D> {
    fn is_terminated(&self) -> bool {
        self.closed
    }
}

impl<D> Stream for Receiver<D> {
    type Item = D;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<D>> {
        if let Some(ref mut receiver) = self.receiver {
            match Pin::new(receiver).poll_next(cx) {
                Poll::Ready(None) => {
                    self.receiver = None;
                    self.closed = true;
                    Poll::Ready(None)
                }
                poll => poll,
            }
        } else if self.closed {
            Poll::Ready(None)
        } else {
            unreachable!();
        }
    }
}

impl<D> Clone for Sender<D> {
    fn clone(&self) -> Sender<D> {
        Sender {
            sender: self.sender.clone(),
            ..*self
        }
    }
}
