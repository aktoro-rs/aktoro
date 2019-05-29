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
use crate::event::EventMessage;
use crate::update;
use crate::update::Updated;
use crate::update::Updater;

pub struct Context<A: raw::Actor> {
    kill: bool,
    status: A::Status,
    ctrler: Controller<A>,
    ctrled: Controlled<A>,
    events: Vec<Box<raw::EventMessage<Actor = A>>>,
    sender: Sender<A>,
    recver: Receiver<A>,
    updter: Updater<A>,
    updted: Option<Updated<A>>,
    _actor: PhantomData<A>,
}

impl<A> raw::Context<A> for Context<A>
where
    A: raw::Actor,
{
    type Controller = Controller<A>;
    type Sender = Sender<A>;
    type Updater = Updater<A>;

    fn new() -> Self {
        let (ctrler, ctrled) = control::new();
        let (sender, recver) = channel::new();
        let (updter, updted) = update::new();

        Context {
            kill: false,
            status: Default::default(),
            ctrler,
            ctrled,
            events: vec![],
            sender,
            recver,
            updter,
            updted: Some(updted),
            _actor: PhantomData,
        }
    }

    fn kill(&mut self) {
        self.kill = true;
    }

    fn killed(&self) -> bool {
        self.kill
    }

    fn emit<E>(&mut self, event: E)
    where
        A: raw::EventHandler<E>,
        E: Send + 'static,
    {
        self.events.push(Box::new(EventMessage::new(event)));
    }

    fn status(&self) -> &A::Status {
        &self.status
    }

    fn update(&mut self, status: A::Status) {
        self.status = status;
    }

    fn controller(&self) -> &Controller<A> {
        &self.ctrler
    }

    fn sender(&self) -> &Sender<A> {
        &self.sender
    }

    fn updated(&mut self) -> Option<Updated<A>> {
        self.updted.take()
    }

    fn controlled(&mut self) -> &mut Controlled<A> {
        &mut self.ctrled
    }

    fn receiver(&mut self) -> &mut Receiver<A> {
        &mut self.recver
    }

    fn updater(&mut self) -> &mut Updater<A> {
        &mut self.updter
    }
}

impl<A> Stream for Context<A>
where
    A: raw::Actor,
{
    type Item = raw::Work<A>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<raw::Work<A>>> {
        let context = self.get_mut();

        if context.kill {
            return Poll::Ready(None);
        }

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
