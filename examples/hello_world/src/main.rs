#![feature(async_await)]

use aktoro::prelude::*;
use aktoro::raw::ActionHandler;
use aktoro::raw::Actor;
use aktoro::raw::Handler;
use aktoro::*;

struct HelloActor;

struct Hello(&'static str);

struct Kill;

impl Actor for HelloActor {
    type Context = Context<Self>;

    type Status = ();
}

impl Handler<Hello> for HelloActor {
    type Output = String;

    fn handle(&mut self, msg: Hello, _: &mut Context<Self>) -> String {
        format!("Hello, {}!", msg.0)
    }
}

impl ActionHandler<Kill> for HelloActor {
    fn handle(&mut self, _: Kill, ctx: &mut Context<Self>) {
        ctx.kill();
    }
}

#[runtime::main]
async fn main() {
    let mut rt = Runtime::init();

    let (ctrler, sender, _) = rt.spawn(HelloActor);

    runtime::spawn(run("World", ctrler, sender));

    rt.wait().await.unwrap();
}

async fn run(
    hello: &'static str,
    mut ctrler: Controller<HelloActor>,
    mut sender: Sender<HelloActor>,
) {
    let msg = Hello(hello);

    let req = sender.send(msg).unwrap();
    let resp = req.await.unwrap();

    println!("{}", resp);

    let req = ctrler.send(Kill).unwrap();
    req.await.unwrap();
}
