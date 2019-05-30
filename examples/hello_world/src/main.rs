#![feature(async_await)]

use aktoro::prelude::*;
use aktoro::*;

struct HelloActor;

struct Hello(&'static str);
struct Bye(&'static str);

struct Kill;

impl Actor for HelloActor {
    type Context = Context<Self>;

    type Status = Status;

    type Error = ();
}

impl Handler<Hello> for HelloActor {
    type Output = String;

    fn handle(&mut self, msg: Hello, _: &mut Context<Self>) -> Result<String, ()> {
        Ok(format!("Hello, {}!", msg.0))
    }
}

impl Handler<Bye> for HelloActor {
    type Output = String;

    fn handle(&mut self, msg: Bye, _: &mut Context<Self>) -> Result<String, ()> {
        Ok(format!("Bye, {}!", msg.0))
    }
}

impl ActionHandler<Kill> for HelloActor {
    type Output = ();

    fn handle(&mut self, _: Kill, ctx: &mut Context<Self>) -> Result<(), ()> {
        ctx.update(Status::Stopped);
        Ok(())
    }
}

#[runtime::main]
async fn main() {
    let mut rt = Runtime::init();

    let (ctrler, sender, _) = rt.spawn(HelloActor).unwrap();

    runtime::spawn(run("World", ctrler, sender));

    rt.wait().await.unwrap();
}

async fn run(
    name: &'static str,
    mut ctrler: Controller<HelloActor>,
    mut sender: Sender<HelloActor>,
) {
    let msg = Hello(name);

    let req = sender.send(msg).unwrap();
    let resp = req.await.unwrap();

    println!("{}", resp);

    let msg = Bye(name);

    let req = sender.send(msg).unwrap();
    let resp = req.await.unwrap();

    println!("{}", resp);

    let req = ctrler.send(Kill).unwrap();
    req.await.unwrap();
}
