#![feature(async_await)]

use aktoro::prelude::*;
use aktoro::raw::Actor;
use aktoro::raw::Handler;
use aktoro::*;

pub struct HelloActor;

pub struct Hello(&'static str);

impl Actor for HelloActor {
    type Context = Context<Self>;

    type Action = ();
    type Event = ();
    type Status = bool;
}

impl Handler<Hello> for HelloActor {
    type Output = String;

    fn handle(&mut self, msg: Hello, _: &mut Context<Self>) -> String {
        format!("Hello, {}!", msg.0)
    }
}

#[runtime::main]
async fn main() {
    let mut rt = Runtime;

    let (_, mut sender) = rt.spawn(HelloActor);

    println!("{:?}", sender.send(Hello("World")).unwrap().await.unwrap());
}
