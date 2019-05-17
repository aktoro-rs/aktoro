#![cfg(feature = "aktoro-runtime")]
#![feature(async_await)]

use aktoro::Actor;
use aktoro::ActorHandled;
use aktoro::Runtime;
use aktoro::RuntimeContext;
use futures_util::FutureExt;

struct Hello;

// `Hello` runs on a `RuntimeContext` and
// handles `String`s sent to it...
impl Actor<RuntimeContext, String> for Hello {
    // ... returning a `String`.
    type Output = String;

    fn handle(&mut self, msg: String, _: &mut RuntimeContext) -> ActorHandled<String> {
        let hello = format!("Hello, {}!", msg);
        async { hello }.boxed()
    }
}

#[runtime::main]
async fn main() {
    // Initialize a new runtime, ...
    let mut runtime = Runtime::init();

    // ... then spawn an `Hello` actor on it, ...
    let (mut hello, mut control) = runtime.spawn(Hello);

    // ... prepare the message to send to it ...
    let msg = String::from("World");

    // ... and, finally, send the message.
    println!("{}", hello.send(msg).await.unwrap());

    // Now, stop the actor ...
    assert!(control.stop().await.is_ok());

    // and wait for the runtime to stop ...
    assert!(runtime.wait().await.is_ok());

    // ... which will be the case immediately
    // because the only actor it was responsible
    // of has already been stopped.
}
