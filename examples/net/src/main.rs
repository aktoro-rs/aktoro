#![feature(async_await)]

use aktoro::prelude::*;
use aktoro::raw;
use futures_util::io::AsyncReadExt;
use futures_util::io::WriteHalf;

struct Client<C: raw::TcpClient> {
    connect: Option<C::Connect>,
    write: Option<WriteHalf<C>>,
}

struct Server<S: raw::TcpServer> {
    tcp: Option<S>,
}

struct Connected<C: raw::TcpClient>(C);

struct Connection<S: raw::TcpServer>(S::Stream);

struct Sent<C: raw::TcpClient>(WriteHalf<C>);

struct Received(Vec<u8>);

impl<C> Actor for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Context = Context<Self>;
    type Status = Status;
    type Error = Error;

    fn started(&mut self, ctx: &mut Self::Context) {
        let connect = self.connect.take().unwrap();

        ctx.wait(connect, |client| Connected(client.unwrap()));
    }
}

impl<S> Actor for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Context = Context<Self>;
    type Status = Status;
    type Error = Error;

    fn started(&mut self, ctx: &mut Self::Context) {
        let tcp = self.tcp.take().unwrap();
        println!("listening on {}", tcp.local_addr().unwrap());

        ctx.subscribe(tcp.into_incoming().unwrap(), |conn| {
            Connection(conn.unwrap())
        });
    }
}

impl<C> Handler<Connected<C>> for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Connected<C>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        let client = msg.0;
        println!("connected to {}", client.peer_addr().unwrap());

        let (_, write) = client.split();
        ctx.write(write, vec![0], |_, write| Sent(write), |_| ());

        Ok(())
    }
}

impl<S> Handler<Connection<S>> for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Connection<S>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        let conn = msg.0;
        println!("new connection from {}", conn.peer_addr().unwrap());

        let (read, _) = conn.split();
        ctx.read(read, 64, |data| Received(data), |_| ());

        Ok(())
    }
}

impl<C> Handler<Sent<C>> for Client<C>
where
    C: raw::TcpClient + 'static,
{
    type Output = ();

    fn handle(&mut self, sent: Sent<C>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("sent data");
        self.write = Some(sent.0);

        ctx.set_status(Status::Stopped);

        Ok(())
    }
}

impl<S> Handler<Received> for Server<S>
where
    S: raw::TcpServer + 'static,
{
    type Output = ();

    fn handle(&mut self, msg: Received, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        println!("received {:?}", msg.0);

        ctx.set_status(Status::Stopped);

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

    let client = Client::<TcpClient> {
        connect: Some(net.tcp_connect("127.0.0.1:5555").unwrap()),
        write: None,
    };

    rt.spawn(client).unwrap();

    rt.wait()
        .await
        .expect("an error occured while waiting for the runtime to stop");
}
