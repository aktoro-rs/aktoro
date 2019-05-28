use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use aktoro_raw as raw;
use futures_core::Stream;

pub(crate) struct Actor<A: raw::Actor> {
    act: A,
    ctx: A::Context,
}

impl<A: raw::Actor> Actor<A> {
    pub(crate) fn new(actor: A, ctx: A::Context) -> Self {
        Actor { act: actor, ctx }
    }
}

impl<A: raw::Actor> Future for Actor<A> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        let actor = self.get_mut();

        loop {
            match Pin::new(&mut actor.ctx).poll_next(ctx) {
                Poll::Ready(Some(work)) => match work {
                    raw::Work::Action(update) => {
                        unimplemented!(); // FIXME
                    }
                    raw::Work::Event(event) => {
                        actor.act.on_event(event, &mut actor.ctx); // FIXME: ret
                    }
                    raw::Work::Message(mut msg) => {
                        msg.handle(&mut actor.act, &mut actor.ctx); // TODO: Result?
                    }
                },
                Poll::Ready(None) => unimplemented!(), // FIXME
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
