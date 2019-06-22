use std::error::Error as StdError;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Eq, PartialEq, Clone, Debug)]
/// An error occuring when calling
/// the `try_clone` method of
/// [`Sender`] or [`Receiver`].
///
/// [`Sender`]: struct.Sender.html#method.try_clone
/// [`Receiver`]: struct.Receiver.html#method.try_clone
pub struct CloneError {
    kind: CloneErrorKind,
}

#[derive(Eq, PartialEq, Clone)]
/// An error occuring while trying
/// to send a message.
pub struct TrySendError<T> {
    kind: SendErrorKind,
    msg: T,
}

#[derive(Eq, PartialEq, Clone, Debug)]
/// An error occuring while trying
/// to receive a message.
pub struct TryRecvError {
    kind: RecvErrorKind,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CloneErrorKind {
    /// The maximum number of sender or
    /// receivers has already been reached.
    Limit,
    /// The sender or receiver is
    /// disconnected from the channel.
    Disconnected,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum SendErrorKind {
    /// The channel is bounded and its
    /// buffer is full.
    Full,
    /// The maximum number of messages
    /// that can be sent over the channel
    /// has been reached.
    Limit,
    /// The sender is disconnected from
    /// the channel.
    Disconnected,
    /// The channel is closed.
    Closed,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum RecvErrorKind {
    /// The receiver is disconnected from
    /// the channel.
    Disconnected,
    /// The channel is closed.
    Closed,
}

impl CloneError {
    /// Creates a new "limit reached"
    /// error.
    pub(crate) fn limit() -> Self {
        CloneError {
            kind: CloneErrorKind::Limit,
        }
    }

    /// Creates a new "disconnected"
    /// error.
    pub(crate) fn disconnected() -> Self {
        CloneError {
            kind: CloneErrorKind::Disconnected,
        }
    }

    /// Whether the error occured
    /// because the maximum number
    /// of senders or receivers has
    /// already been reached.
    pub fn is_limit(&self) -> bool {
        self.kind == CloneErrorKind::Limit
    }

    /// Whether the error occured
    /// because the sender or receiver
    /// is disconnected from the channel.
    pub fn is_disconnected(&self) -> bool {
        self.kind == CloneErrorKind::Disconnected
    }
}

impl<T> TrySendError<T> {
    /// Creates a new "channel full"
    /// error.
    pub(crate) fn full(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Full,
            msg,
        }
    }

    /// Creates a new "limit reached"
    /// error.
    pub(crate) fn limit(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Limit,
            msg,
        }
    }

    /// Creates a new "disconnected"
    /// error.
    pub(crate) fn disconnected(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Disconnected,
            msg,
        }
    }

    /// Creates a new "channel closed"
    /// error.
    pub(crate) fn closed(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Closed,
            msg,
        }
    }

    /// Whether the error occured because
    /// the channel's buffer is full.
    pub fn is_full(&self) -> bool {
        self.kind == SendErrorKind::Full
    }

    /// Whether the error occured because
    /// the maximum number of messages
    /// that can be sent over the channel
    /// has already been reached.
    pub fn is_limit(&self) -> bool {
        self.kind == SendErrorKind::Limit
    }

    /// Whether the error occured because
    /// the sender is disconnected from
    /// the channel.
    pub fn is_disconnected(&self) -> bool {
        self.kind == SendErrorKind::Disconnected
    }

    /// Whether the error occured because
    /// the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.kind == SendErrorKind::Closed
    }

    /// Gets a reference to the message
    /// that the sender was trying to
    /// send.
    pub fn msg(&self) -> &T {
        &self.msg
    }

    /// Gets the message that the sender
    /// was trying to send, consuming
    /// the error.
    pub fn into_msg(self) -> T {
        self.msg
    }
}

impl TryRecvError {
    /// Creates a new "disconncted" error.
    pub(crate) fn disconnected() -> Self {
        TryRecvError {
            kind: RecvErrorKind::Disconnected,
        }
    }

    /// Creates a new "channel closed"
    /// error.
    pub(crate) fn closed() -> Self {
        TryRecvError {
            kind: RecvErrorKind::Closed,
        }
    }

    /// Whether the error occured because
    /// the receiver is disconnected from
    /// the channel.
    pub fn is_disconnected(&self) -> bool {
        self.kind == RecvErrorKind::Disconnected
    }

    /// Whether the error occured because
    /// the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.kind == RecvErrorKind::Closed
    }
}

impl StdError for CloneError {}

impl<T> StdError for TrySendError<T> {}

impl StdError for TryRecvError {}

impl Display for CloneError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.kind {
            CloneErrorKind::Limit => write!(fmt, "clone failed because limit reached",),
            CloneErrorKind::Disconnected => {
                write!(fmt, "clone failed because already disconnected",)
            }
        }
    }
}

impl<T> Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.kind {
            SendErrorKind::Full => write!(fmt, "send failed because channel full",),
            SendErrorKind::Limit => write!(fmt, "send failed because limit reached",),
            SendErrorKind::Disconnected => write!(fmt, "send failed because already disconnected",),
            SendErrorKind::Closed => write!(fmt, "send failed because channel closed",),
        }
    }
}

impl Display for TryRecvError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.kind {
            RecvErrorKind::Disconnected => {
                write!(fmt, "receive failed because already disconnected",)
            }
            RecvErrorKind::Closed => write!(fmt, "receive failed because channel closed",),
        }
    }
}

impl<T> Debug for TrySendError<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("TrySendError")
            .field("kind", &self.kind)
            .finish()
    }
}
