use crate::actor::Actor;

/// TODO: documentation
pub trait Handler<M: Send>: Actor {
    /// TODO: documentation
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Result<Output<Self>, Self::Error>;
}

/// TODO: documentation
pub struct Output<A>
where
    A: Actor,
{
    status: Option<A::Status>,
}

impl<A> super::Output<A> for Output<A>
where
    A: Actor,
{
    /// TODO: documentation
    fn status(&mut self, status: A::Status) {
        self.status = Some(status);
    }
}

impl<A> Default for Output<A>
where
    A: Actor,
{
    fn default() -> Self {
        Output {
            status: None,
        }
    }
}
