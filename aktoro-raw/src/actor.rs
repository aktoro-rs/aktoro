use crate::context::Context;

pub trait Actor: Unpin + Send + Sized + 'static {
    type Context: Context<Self>;

    type Action: Action + Send;
    type Event: Event + Send;
    type Status: Status + Send;

    fn on_action(&mut self, action: Self::Action, ctx: &mut Self::Context) -> Self::Status {
        Status::running()
    }

    fn on_event(&mut self, event: Self::Event, ctx: &mut Self::Context) -> Self::Status {
        Status::running()
    }
}

pub trait Action {} // TODO

pub trait Event {} // TODO

pub trait Status {
    fn running() -> Self;
} // TODO

pub trait Handler<M>: Actor {
    type Output: Send;

    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Output;
}

impl Action for () {}

impl Event for () {}

impl Status for bool {
    fn running() -> bool {
        true
    }
}
