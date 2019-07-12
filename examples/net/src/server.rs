use aktoro::prelude::*;
use aktoro::raw;
use futures_util::AsyncReadExt;
use futures_util::StreamExt;

use crate::agent::Agent;

struct Connection<S: raw::TcpServer>(S::Stream);

struct Died;

pub(crate) struct Server<S: raw::TcpServer> {
    pub(crate) tcp: Option<S>,
    pub(crate) cancellable: Option<Cancellable<TcpServerIncoming<'static, S>>>,
}

impl<S> Actor for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Context = Context<Self, Runtime>;
    type Status = Status;
    type Error = Error;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("server({}): started", ctx.actor_id());
        println!("server({}): listening", ctx.actor_id());

        let tcp = self.tcp.take().unwrap();
        self.cancellable = Some(
            ctx.subscribe(Box::pin(tcp.into_incoming().unwrap()), |conn| {
                Connection(conn.unwrap())
            }),
        );
    }
}

impl<S> Handler<Connection<S>> for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Connection<S>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("server({}): new connection", ctx.actor_id());

        let actor_id = ctx.actor_id();
        println!("server({}): closing", actor_id);
        ctx.blocking_wait(Box::pin(self.cancellable.take().unwrap().cancel()), move |_| {
            println!("server({}): closed", actor_id);
            ()
        });

        let (read, write) = msg.0.split();
        let spawned = ctx.spawn(Agent {
            read: Some(Box::pin(read)),
            write: Some(Box::pin(write)),
            cancellable: None,
        }).unwrap();

        ctx.subscribe(spawned.boxed(), |_| Died);

        Ok(())
    }
}

impl<S> Handler<Died> for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Output = ();

    fn handle(&mut self, _: Died, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("server({}): agent died; remaining: {:?}", ctx.actor_id(), ctx.actors());

        ctx.set_status(Status::Dead);

        Ok(())
    }
}
