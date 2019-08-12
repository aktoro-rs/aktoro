use crate::actor::Actor;

/// TODO: documentation
pub trait Handler<A: Send>: Actor {
    /// TODO: documentation
    type Output: Unpin + Send;

    /// TODO: documentation
    fn handle(&mut self, action: A, ctx: &mut Self::Context) -> Result<Output<Self, Self::Output>, Self::Error>;
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

impl<A, O> super::Output<A> for Output<A, O>
where
    A: Actor,
    O: Unpin + Send,
{
    /// TODO: documentation
    fn status(&mut self, status: A::Status) {
        self.status = Some(status);
    }
}

impl<A, O> Output<A, O>
where
    A: Actor,
    O: Unpin + Send,
{
    /// TODO: documentation
    pub fn output(&mut self, output: O) {
        self.output = Some(output);
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
