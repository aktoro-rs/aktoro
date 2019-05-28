use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_raw as raw;
use futures_core::Stream;

use crate::channel;
use crate::channel::Receiver;
use crate::channel::Sender;
use crate::control;
use crate::control::Controlled;
use crate::control::Controller;

pub struct Context<A: raw::Actor> {
    ctrler: Controller<A>,
    ctrled: Controlled<A>,
    sender: Sender<A>,
    recver: Receiver<A>,
    _actor: PhantomData<A>,
}

impl<A> raw::Context<A> for Context<A>
where
    A: raw::Actor,
{
    type Controller = Controller<A>;
    type Sender = Sender<A>;

    fn new() -> Self {
        let (ctrler, ctrled) = control::new();
        let (sender, recver) = channel::new();

        Context {
            ctrler,
            ctrled,
            sender,
            recver,
            _actor: PhantomData,
        }
    }

    fn controller(&self) -> Controller<A> {
        self.ctrler.clone()
    }

    fn sender(&self) -> Sender<A> {
        self.sender.clone()
    }
}

impl<A> Stream for Context<A>
where
    A: raw::Actor,
{
    type Item = raw::Work<A>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<raw::Work<A>>> {
        let context = self.get_mut();

        match Pin::new(&mut context.ctrled).poll_next(ctx) {
            Poll::Ready(Some(update)) => {
                return Poll::Ready(Some(raw::Work::Action(update)));
            }
            Poll::Ready(None) => unimplemented!(), // FIXME
            Poll::Pending => (),
        }

        match Pin::new(&mut context.recver).poll_next(ctx) {
            Poll::Ready(Some(msg)) => {
                return Poll::Ready(Some(raw::Work::Message(msg)));
            }
            Poll::Ready(None) => unimplemented!(), // FIXME
            Poll::Pending => (),
        }

        // TODO: event
        // TODO: local actions

        Poll::Pending
    }
}
