use std::error::Error as StdError;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CloneError {
    kind: CloneErrorKind,
}

#[derive(Eq, PartialEq, Clone)]
pub struct TrySendError<T> {
    kind: SendErrorKind,
    msg: T,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TryRecvError {
    kind: RecvErrorKind,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CloneErrorKind {
    Limit,
    Disconnected,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum SendErrorKind {
    Full,
    Limit,
    Disconnected,
    Closed,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum RecvErrorKind {
    Disconnected,
    Closed,
}

impl CloneError {
    pub(crate) fn limit() -> Self {
        CloneError {
            kind: CloneErrorKind::Limit,
        }
    }

    pub(crate) fn disconnected() -> Self {
        CloneError {
            kind: CloneErrorKind::Disconnected,
        }
    }

    pub fn is_full(&self) -> bool {
        self.kind == CloneErrorKind::Limit
    }

    pub fn is_disconnected(&self) -> bool {
        self.kind == CloneErrorKind::Disconnected
    }
}

impl<T> TrySendError<T> {
    pub(crate) fn full(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Full,
            msg,
        }
    }

    pub(crate) fn limit(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Limit,
            msg,
        }
    }

    pub(crate) fn disconnected(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Disconnected,
            msg,
        }
    }

    pub(crate) fn closed(msg: T) -> Self {
        TrySendError {
            kind: SendErrorKind::Closed,
            msg,
        }
    }

    pub fn is_full(&self) -> bool {
        self.kind == SendErrorKind::Full
    }

    pub fn is_limit(&self) -> bool {
        self.kind == SendErrorKind::Limit
    }

    pub fn is_disconnected(&self) -> bool {
        self.kind == SendErrorKind::Disconnected
    }

    pub fn is_closed(&self) -> bool {
        self.kind == SendErrorKind::Closed
    }

    pub fn msg(&self) -> &T {
        &self.msg
    }

    pub fn into_msg(self) -> T {
        self.msg
    }
}

impl TryRecvError {
    pub(crate) fn disconnected() -> Self {
        TryRecvError {
            kind: RecvErrorKind::Disconnected,
        }
    }

    pub(crate) fn closed() -> Self {
        TryRecvError {
            kind: RecvErrorKind::Closed,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.kind == RecvErrorKind::Disconnected
    }

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
