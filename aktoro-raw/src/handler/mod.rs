use crate::actor::Actor;
use crate::actor::Status;

/// TODO: module documentation

/// TODO: documentation
pub mod action;

/// TODO: documentation
pub mod event;

/// TODO: documentation
pub mod message;

pub trait Output<A>: Default + Unpin + Send
where
    A: Actor,
{
    /// TODO: documentation
    fn new() -> Self {
        Default::default()
    }

    /// TODO: documentation
    fn status(&mut self, status: A::Status);

    /// TODO: documentation
    fn stop(&mut self) {
        self.status(A::Status::stopping());
    }

    /// TODO: documentation
    fn force_stop(&mut self) {
        self.status(A::Status::stopping());
    }

    /// TODO: documentation
    fn kill(&mut self) {
        self.status(A::Status::dead());
    }

    /// TODO: documentation
    fn panic(&mut self, error: A::Error) {
        self.status(A::Status::panicked(error));
    }
}
