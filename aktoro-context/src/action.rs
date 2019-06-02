use aktoro_raw as raw;

use crate::respond::Respond;

pub(crate) struct Action<A, D>
where
    A: raw::ActionHandler<D>,
    D: Send + 'static,
{
    action: Option<D>,
    resp: Respond<A::Output>,
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
                resp: resp.0,
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
        if let Some(action) = self.action.take() {
            self.resp.respond(actor.handle(action, ctx)?);
        }

        Ok(())
    }
}
