use crate::actor::Actor;

pub trait Action: Send {
    type Actor: Actor;

    fn handle(
        &mut self,
        actor: &mut Self::Actor,
        ctx: &mut <Self::Actor as Actor>::Context,
    ) -> Result<(), <Self::Actor as Actor>::Error>;
}

pub trait ActionHandler<A: Send>: Actor {
    type Output: Send;

    /// Handles the action, returning a result
    /// eventually containing the action's output.
    fn handle(&mut self, action: A, ctx: &mut Self::Context) -> Result<Self::Output, Self::Error>;
}
