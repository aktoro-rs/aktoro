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
    AlreadyRemoved(u64),
    Std(Box<StdError + Send>),
    Multiple(Vec<Error>),
}

impl Error {
    pub(crate) fn already_removed(id: u64) -> Self {
        Error {
            kind: ErrorKind::AlreadyRemoved(id),
        }
    }

    pub(crate) fn std<S>(err: S) -> Self
    where
        S: StdError + Send + 'static,
    {
        Error {
            kind: ErrorKind::Std(Box::new(err)),
        }
    }

    pub(crate) fn multiple(errors: Vec<Error>) -> Self {
        Error {
            kind: ErrorKind::Multiple(errors),
        }
    }

    pub(crate) fn add_err(self, err: Error) -> Error {
        let mut error;
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

    pub(crate) fn add_res<O>(self, res: Result<O, Error>) -> Error {
        match res {
            Ok(_) => self,
            Err(err) => self.add_err(err),
        }
    }

    pub fn is_already_removed(&self) -> bool {
        if let ErrorKind::AlreadyRemoved(_) = self.kind {
            true
        } else {
            false
        }
    }

    pub fn is_multiple(&self) -> bool {
        if let ErrorKind::Multiple(_) = self.kind {
            true
        } else {
            false
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn into_kind(self) -> ErrorKind {
        self.kind
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::AlreadyRemoved(id) => {
                write!(fmt, "actor ({}) already removed from list", id,)
            }
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
