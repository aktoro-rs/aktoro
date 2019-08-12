use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {

}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self.kind {

        }
    }
}
