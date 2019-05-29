use std::marker::PhantomData;

use aktoro_channel::once;
use aktoro_raw as raw;

pub(crate) struct ActionMessage<A, D>
where
    A: raw::ActionHandler<D>,
    D: Send + 'static,
{
    action: Option<D>,
    resp: once::Sender<()>,
    _actor: PhantomData<A>,
}

pub(crate) fn new<A, D>(action: D) -> (ActionMessage<A, D>, once::Receiver<()>)
where
    A: raw::ActionHandler<D>,
    D: Send + 'static,
{
    let (sender, recver) = once::new();

    (
        ActionMessage {
            action: Some(action),
            resp: sender,
            _actor: PhantomData,
        },
        recver,
    )
}

impl<A, D> raw::ActionMessage for ActionMessage<A, D>
where
    A: raw::ActionHandler<D>,
    D: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) {
        if let Some(action) = self.action.take() {
            self.resp.send(actor.handle(action, ctx)); // TODO: handle
        }
    }
}
