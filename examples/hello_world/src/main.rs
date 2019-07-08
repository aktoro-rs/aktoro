#![feature(async_await)]

use std::task::Poll;

use aktoro::prelude::*;
use futures_util::poll;

struct HelloActor;

struct Hello(&'static str);
struct Bye(&'static str);

struct Kill;

impl Actor for HelloActor {
    type Context = Context<Self>;
    type Status = Status;
    type Error = Error;
}

impl Handler<Hello> for HelloActor {
    type Output = String;

    fn handle(&mut self, msg: Hello, _: &mut Context<Self>) -> Result<String, Error> {
        Ok(format!("Hello, {}!", msg.0))
    }
}

impl Handler<Bye> for HelloActor {
    type Output = String;

    fn handle(&mut self, msg: Bye, _: &mut Context<Self>) -> Result<String, Error> {
        Ok(format!("Bye, {}!", msg.0))
    }
}

impl ActionHandler<Kill> for HelloActor {
    type Output = ();

    fn handle(&mut self, _: Kill, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.set_status(Status::Stopped);
        Ok(())
    }
}

#[runtime::main]
async fn main() {
    let mut rt = Runtime::new();

    let spawned = rt.spawn(HelloActor).unwrap();

    let mut run = runtime::spawn(run("World", spawned));
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

async fn run(name: &'static str, mut spawned: Spawned<HelloActor>) {
    let msg = Hello(name);

    let req = spawned.try_send_msg(msg).unwrap();
    let resp = req.await.unwrap();

    println!("{}", resp);

    let msg = Bye(name);

    let req = spawned.try_send_msg(msg).unwrap();
    let resp = req.await.unwrap();

    println!("{}", resp);

    let req = spawned.try_send_action(Kill).unwrap();
    req.await.unwrap();
}
