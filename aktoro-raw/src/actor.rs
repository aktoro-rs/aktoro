use futures_core::future::BoxFuture;

use crate::context::Context;

pub trait Actor: Sized {
    type Context: Context<Self>;
}

pub trait Handler<M>: Actor {
    type Output;

    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> BoxFuture<Self::Output>;
}
