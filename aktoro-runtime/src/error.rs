use std::error::Error as StdError;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    /// Returns a `Box` contaning any error type
    /// that implements the [`Error`] trait.
    ///
    /// [`Error`]: https://doc.rust-lang.org/std/error/trait.Error.html
    Std(Box<dyn StdError + Send>),
    /// Multiple errors occured.
    Multiple(Vec<Error>),
}

impl Error {
    /// Creates a new boxed error.
    pub(crate) fn std<S>(err: S) -> Self
    where
        S: StdError + Send + 'static,
    {
        Error {
            kind: ErrorKind::Std(Box::new(err)),
        }
    }

    /// Creates a new "multiple errors" error.
    pub(crate) fn multiple(errors: Vec<Error>) -> Self {
        Error {
            kind: ErrorKind::Multiple(errors),
        }
    }

    /// Add an error to the current error returning
    /// a [`Error::Multiple`] error containing the
    /// two errors.
    ///
    /// [`Error::Multiple`]: enum.ErrorKind.html#variant.Multiple
    pub(crate) fn add_err(self, err: Error) -> Error {
        let error;
        match (self.kind, err.kind) {
            (ErrorKind::Multiple(mut errs), ErrorKind::Multiple(mut errs_)) => {
                errs.append(&mut errs_);
                error = Error::multiple(errs);
            }
            (ErrorKind::Multiple(mut errs), err) => {
                errs.push(err.into());
                error = Error::multiple(errs);
            }
            (err, ErrorKind::Multiple(mut errs)) => {
                errs.push(err.into());
                error = Error::multiple(errs);
            }
            (err, err_) => {
                return Error {
                    kind: ErrorKind::Multiple(vec![err.into(), err_.into()]),
                }
            }
        }

        error
    }

    /// If `res` if an `Err`, calls [`add_err`] with it,
    /// or returns the current error otherwise.
    ///
    /// [`add_err`]: #method.add_err
    pub(crate) fn add_res<O>(self, res: Result<O, Error>) -> Error {
        match res {
            Ok(_) => self,
            Err(err) => self.add_err(err),
        }
    }

    /// Whether multiple errors occured.
    pub fn is_multiple(&self) -> bool {
        if let ErrorKind::Multiple(_) = self.kind {
            true
        } else {
            false
        }
    }

    /// Returns a reference to the error's kind.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Returns the error's kind, consuming the
    /// error.
    pub fn into_kind(self) -> ErrorKind {
        self.kind
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Std(err) => write!(fmt, "{}", err),
            ErrorKind::Multiple(_) => write!(fmt, "multiple errors",),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl<S> From<Box<S>> for Error
where
    S: StdError + Send + 'static,
{
    fn from(err: Box<S>) -> Error {
        Error {
            kind: ErrorKind::Std(err),
        }
    }
}
