use aktoro_raw as raw;

use crate::respond::Respond;

/// A wrapper around an action that an actor should
/// handle (this is used to allow generalization).
pub(crate) struct Action<A, D>
where
    A: raw::ActionHandler<D>,
    D: Send + 'static,
{
    action: Option<D>,
    resp: Option<Respond<A::Output>>,
}

impl<A, D> Action<A, D>
where
    A: raw::ActionHandler<D>,
    D: Send + 'static,
{
    pub(crate) fn new(action: D) -> (Self, Respond<A::Output>) {
        let resp = Respond::new();

        (
            Action {
                action: Some(action),
                resp: Some(resp.0),
            },
            resp.1,
        )
    }
}

impl<A, D> raw::Action for Action<A, D>
where
    A: raw::ActionHandler<D>,
    D: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) -> Result<(), A::Error> {
        // If the action hasn't already been handled,
        // we do so and return the result.
        if let Some(action) = self.action.take() {
            self.resp
                .take()
                .unwrap()
                .respond(actor.handle(action, ctx)?);
        }

        Ok(())
    }
}
