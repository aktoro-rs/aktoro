#![feature(async_await)]

use std::task::Poll;

use aktoro::prelude::*;
use aktoro::raw;
use futures_util::poll;

struct Server<S: raw::TcpServer> {
    tcp: Option<S>,
}

struct Connection<S: raw::TcpServer> {
    stream: S::Stream,
}

impl<S> Actor for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Context = Context<Self>;
    type Status = Status;
    type Error = Error;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("started");
        ctx.subscribe(self.tcp.take().unwrap().into_incoming().unwrap(), |conn| {
            Connection {
                stream: conn.unwrap(),
            }
        });
    }
}

impl<S> Handler<Connection<S>> for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Connection<S>, ctx: &mut Context<Self>) -> Result<(), Self::Error> {
        println!("new connection");
        Ok(())
    }
}

#[runtime::main]
async fn main() {
    let mut rt = Runtime::new();
    let net = rt.net();

    let server = Server {
        tcp: Some(net.tcp_bind("127.0.0.1:5555").unwrap()),
    };

    rt.spawn(server).unwrap();

    let mut run = runtime::spawn(run(net));
    let mut wait = rt.wait();

    loop {
        if let Poll::Ready(_) = poll!(&mut run) {
            break;
        }

        if let Poll::Ready(res) = poll!(&mut wait) {
            res.expect("an error occured while waiting for the runtime to stop");
            break;
        }
    }
}

async fn run<N: raw::NetworkManager>(net: N) {
    loop {}
}
