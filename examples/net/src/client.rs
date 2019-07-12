use std::pin::Pin;

use aktoro::prelude::*;
use aktoro::raw;
use futures_util::io::WriteHalf;
use futures_util::AsyncReadExt;

use crate::Received;
use crate::Sent;

struct Connected<C: raw::TcpClient>(C);

struct ConnectedErr<C: raw::TcpClient>(<C as raw::TcpClient>::Error);

pub(crate) struct Client<C: raw::TcpClient> {
    pub(crate) connect: Option<Pin<Box<C::Connect>>>,
    pub(crate) closed_connect: Option<Pin<Box<C::Connect>>>,
    pub(crate) write: Option<Pin<Box<WriteHalf<C>>>>,
}

impl<C> Actor for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Context = Context<Self, Runtime>;
    type Status = Status;
    type Error = Error;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("client({}): started", ctx.actor_id());
        println!("client({}): connecting", ctx.actor_id());

        let connect = self.connect.take().unwrap();
        ctx.wait(connect, |res| Connected(res.unwrap()));
    }
}

impl<C> Handler<Connected<C>> for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Connected<C>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("client({}): connected", ctx.actor_id());

        let (read, write) = msg.0.split();
        ctx.read(Box::pin(read), 64, Received, |_| ());
        ctx.write(Box::pin(write), vec![0], |_, write| Sent(write), |_| ());

        Ok(())
    }
}

impl<C> Handler<ConnectedErr<C>> for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: ConnectedErr<C>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!(
            "client({}): failed connecting; reason={}",
            ctx.actor_id(),
            msg.0
        );

        ctx.set_status(Status::Dead);

        Ok(())
    }
}

impl<C> Handler<Sent<C>> for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Sent<C>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("client({}): sent data", ctx.actor_id());

        self.write = Some(msg.0);

        Ok(())
    }
}

impl<C> Handler<Received> for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Received, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!(
            "client({}): received data; data={:?}",
            ctx.actor_id(),
            msg.0
        );

        println!("client({}): connecting (failing)", ctx.actor_id());
        let connect = self.closed_connect.take().unwrap();
        ctx.wait(connect, |res| ConnectedErr(res.err().unwrap()));

        Ok(())
    }
}
