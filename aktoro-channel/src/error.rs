use std::error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

/// TODO: documentation
pub struct Error<T = ()> {
    kind: ErrorKind<T>,
}

/// TODO: documentation
pub enum ErrorKind<T = ()> {
    /// TODO: documentation
    Full(T),
    /// TODO: documentation
    SenderLimit,
    /// TODO: documentation
    ReceiverLimit,
    /// TODO: documentation
    MessageLimit(T),
    /// TODO: documentation
	Disconnected(Option<T>),
    /// TODO: documentation
    Closed(Option<T>),
    /// TODO: documentation
    Unsupported(Option<T>),
}

/// TODO(methods): is_*
/// TODO(method): kind
/// TODO(method): into_kind
impl<T> Error<T> {
    /// TODO: documentation
    pub(crate) fn full(value: T) -> Error<T> {
        Error {
            kind: ErrorKind::Full(value),
        }
    }

    /// TODO: documentation
	pub(crate) fn sender_limit() -> Error<T> {
		Error {
			kind: ErrorKind::SenderLimit,
		}
	}

    /// TODO: documentation
	pub(crate) fn recver_limit() -> Error<T> {
		Error {
			kind: ErrorKind::ReceiverLimit,
		}
	}

    /// TODO: documentation
    pub(crate) fn msg_limit(value: T) -> Error<T> {
        Error {
            kind: ErrorKind::MessageLimit(value),
        }
    }

    /// TODO: documentation
    pub(crate) fn disconnected(value: Option<T>) -> Error<T> {
        Error {
            kind: ErrorKind::Disconnected(value),
        }
    }

    /// TODO: documentation
    pub(crate) fn closed(value: Option<T>) -> Error<T> {
        Error {
            kind: ErrorKind::Closed(value),
        }
    }

    /// TODO: documentation
    pub(crate) fn unsupported(value: Option<T>) -> Error<T> {
        Error {
            kind: ErrorKind::Unsupported(value)
        }
    }

	/// TODO: documentation
	pub(crate) fn map<M, U>(self, mapper: M) -> Error<U>
	where
		M: Fn(T) -> U,
	{
		match self.kind {
			ErrorKind::Full(value) => Error::full(mapper(value)),
			ErrorKind::SenderLimit => Error::sender_limit(),
			ErrorKind::ReceiverLimit => Error::recver_limit(),
			ErrorKind::MessageLimit(value) => Error::msg_limit(mapper(value)),
			ErrorKind::Disconnected(Some(value)) => Error::disconnected(Some(mapper(value))),
			ErrorKind::Disconnected(None) => Error::disconnected(None),
			ErrorKind::Closed(Some(value)) => Error::closed(Some(mapper(value))),
			ErrorKind::Closed(None) => Error::closed(None),
			ErrorKind::Unsupported(Some(value)) => Error::unsupported(Some(mapper(value))),
			ErrorKind::Unsupported(None) => Error::unsupported(None),
		}
	}
}

impl<T> error::Error for Error<T> {}

impl<T> Display for Error<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Ok(()) // TODO
    }
}

impl<T> Debug for Error<T> {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Ok(()) // TODO
	}
}

impl<T> Debug for ErrorKind<T> {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Ok(()) // TODO
	}
}
