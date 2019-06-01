use std::fmt;

#[derive(Eq, PartialEq, Clone)]
pub struct CloneError {
    kind: CloneErrorKind,
}

#[derive(Eq, PartialEq, Clone)]
pub struct TrySendError<T> {
    kind: SendErrorKind,
    msg: T,
}

#[derive(Eq, PartialEq, Clone)]
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

impl fmt::Debug for CloneError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CloneError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Debug for TryRecvError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("TryRecvError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("TrySendError")
            .field("kind", &self.kind)
            .finish()
    }
}
