use aktoro_raw as raw;

use crate::respond::Respond;

/// A wrapper around a message that an actor should
/// handle (this is used to allow generalization).
pub(crate) struct Message<A, M>
where
    A: raw::Handler<M>,
    M: Send + 'static,
{
    msg: Option<M>,
    resp: Option<Respond<A::Output>>,
}

impl<A, M> Message<A, M>
where
    A: raw::Handler<M>,
    M: Send + 'static,
{
    pub(crate) fn new(msg: M) -> (Self, Respond<A::Output>) {
        let resp = Respond::new();

        (
            Message {
                msg: Some(msg),
                resp: Some(resp.0),
            },
            resp.1,
        )
    }
}

impl<A, M> raw::Message for Message<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) -> Result<(), A::Error> {
        // If the message hasn't already been handled,
        // we do so and return the result.
        if let Some(msg) = self.msg.take() {
            self.resp.take().unwrap().respond(actor.handle(msg, ctx)?);
        }

        Ok(())
    }
}
