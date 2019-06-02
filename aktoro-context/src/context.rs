use std::collections::VecDeque;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::error::TrySendError;
use aktoro_raw as raw;
use aktoro_raw::Updater as RawUpdater;
use futures_core::Stream;

use crate::channel;
use crate::channel::Receiver;
use crate::channel::Sender;
use crate::control;
use crate::control::Controlled;
use crate::control::Controller;
use crate::event::Event;
use crate::update;
use crate::update::Updated;
use crate::update::Updater;

pub struct Context<A: raw::Actor> {
    status: A::Status,
    ctrler: Controller<A>,
    ctrled: Controlled<A>,
    update: bool,
    events: VecDeque<Box<dyn raw::Event<Actor = A>>>,
    sender: Sender<A>,
    recver: Receiver<A>,
    updter: Updater<A>,
    updted: Option<Updated<A>>,
}

impl<A> raw::Context<A> for Context<A>
where
    A: raw::Actor,
{
    type Controller = Controller<A>;
    type Sender = Sender<A>;
    type Updater = Updater<A>;

    fn new() -> Context<A> {
        let (ctrler, ctrled) = control::new();
        let (sender, recver) = channel::new();
        let (updter, updted) = update::new();

        Context {
            status: A::Status::default(),
            ctrler,
            ctrled,
            update: false,
            events: VecDeque::new(),
            sender,
            recver,
            updter,
            updted: Some(updted),
        }
    }

    fn emit<E>(&mut self, event: E)
    where
        A: raw::EventHandler<E>,
        E: Send + 'static,
    {
        self.events.push_back(Box::new(Event::new(event)));
    }

    fn status(&self) -> &A::Status {
        &self.status
    }

    fn set_status(&mut self, status: A::Status) {
        if self.status != status {
            self.status = status;
            self.update = true;
        }
    }

    fn update(&mut self) -> Result<(), TrySendError<A::Status>> {
        self.update = false;
        self.updter.try_send(self.status.clone())
    }

    fn controller(&self) -> &Controller<A> {
        &self.ctrler
    }

    fn sender(&self) -> &Sender<A> {
        &self.sender
    }

    fn updated_ref(&mut self) -> Option<&mut Updated<A>> {
        self.updted.as_mut()
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

        match Pin::new(&mut context.ctrled).poll_next(ctx) {
            Poll::Ready(Some(update)) => {
                return Poll::Ready(Some(raw::Work::Action(update)));
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => (),
        }

        if context.update {
            context.update = false;
            return Poll::Ready(Some(raw::Work::Update));
        }

        if let Some(event) = context.events.pop_front() {
            return Poll::Ready(Some(raw::Work::Event(event)));
        }

        match Pin::new(&mut context.recver).poll_next(ctx) {
            Poll::Ready(Some(msg)) => {
                return Poll::Ready(Some(raw::Work::Message(msg)));
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => (),
        }

        Poll::Pending
    }
}
