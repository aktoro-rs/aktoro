use crate::actor::Actor;
use crate::actor::Status;

/// TODO: documentation
pub struct Handled<A>
where
    A: Actor,
{
    status: Option<A::Status>,
}

/// TODO: documentation
pub struct Output<A, O>
where
    A: Actor,
    O: Send,
{
    status: Option<A::Status>,
    output: Option<O>,
}

/// TODO: documentation
pub trait HandleRes<A>: Default + Send
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

impl<A, O> Output<A, O>
where
    A: Actor,
    O: Send,
{
    pub fn output(&mut self, output: O) {
        self.output = Some(output);
    }
}

impl<A> HandleRes<A> for Handled<A>
where
    A: Actor,
{
    fn status(&mut self, status: A::Status) {
        self.status = Some(status);
    }
}

impl<A, O> HandleRes<A> for Output<A, O>
where
    A: Actor,
    O: Send,
{
    fn status(&mut self, status: A::Status) {
        self.status = Some(status);
    }
}


impl<A> Default for Handled<A>
where
    A: Actor,
{
    fn default() -> Self {
        Handled { status: None }
    }
}

impl<A, O> Default for Output<A, O>
where
    A: Actor,
    O: Send,
{
    fn default() -> Self {
        Output {
            status: None,
            output: None,
        }
    }
}
