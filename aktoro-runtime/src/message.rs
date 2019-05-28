use aktoro_channel::once;
use aktoro_raw as raw;

pub(crate) struct Message<A, M>
where
    A: raw::Handler<M>,
{
    msg: Option<M>,
    resp: once::Sender<A::Output>,
}

impl<A, M> Message<A, M>
where
    A: raw::Handler<M>,
{
    pub(crate) fn new(msg: M) -> (Self, once::Receiver<A::Output>) {
        let (sender, recver) = once::new();

        (
            Message {
                msg: Some(msg),
                resp: sender,
            },
            recver,
        )
    }
}

impl<A, M> raw::Message for Message<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) {
        if let Some(msg) = self.msg.take() {
            self.resp.send(actor.handle(msg, ctx)).ok().unwrap(); // FIXME
        }
    }
}
