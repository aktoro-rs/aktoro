use std::pin::Pin;

use aktoro::prelude::*;
use aktoro::raw;
use futures_util::io::ReadHalf;
use futures_util::io::WriteHalf;

use crate::Received;
use crate::Sent;

struct Closed;

pub(crate) struct Agent<S: raw::TcpStream> {
    pub(crate) read: Option<Pin<Box<ReadHalf<S>>>>,
    pub(crate) write: Option<Pin<Box<WriteHalf<S>>>>,
    pub(crate) cancellable: Option<Cancellable<ReadHalf<S>>>,
}

impl<S> Actor for Agent<S>
where
    S: raw::TcpStream + 'static,
{
    type Context = Context<Self, Runtime>;
    type Status = Status;
    type Error = Error;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("agent({}): started", ctx.actor_id());

        self.cancellable = Some(
            ctx.read(self.read.take().unwrap(), 64, Received, |_| ()),
        );
    }
}

impl<S> Handler<Sent<S>> for Agent<S>
where
    S: raw::TcpStream + 'static,
{
    type Output = ();

    fn handle(&mut self, _: Sent<S>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("agent({}): sent data", ctx.actor_id());

        println!("agent({}): closing stream", ctx.actor_id());
        ctx.wait(Box::pin(self.cancellable.take().unwrap().cancel()), |_| Closed);

        Ok(())
    }
}

impl<S> Handler<Received> for Agent<S>
where
    S: raw::TcpStream + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Received, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("agent({}): received data; data={:?}", ctx.actor_id(), msg.0);

        ctx.write(self.write.take().unwrap(), vec![0], |_, write| Sent(write), |_| ());

        Ok(())
    }
}

impl<S> Handler<Closed> for Agent<S>
where
    S: raw::TcpStream + 'static,
{
    type Output = ();

    fn handle(&mut self, _: Closed, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("agent({}): closed stream", ctx.actor_id());

        ctx.set_status(Status::Dead);

        Ok(())
    }
}
