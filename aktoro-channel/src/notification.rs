use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use aktoro_raw as raw;

use crate::error::Error;
use crate::once;
use crate::once::Receiver;
use crate::once::Sender;

/// TODO: documentation
pub(crate) fn new() -> (Notify, Received) {
    let (sender, recver) = once::new();
    (Notify { sender, }, Received { recver, })
}

/// TODO/ documentation
pub struct Notify {
    /// TODO/ documentation
    sender: Sender<()>,
}

/// TODO/ documentation
pub struct Received {
    /// TODO/ documentation
    recver: Receiver<()>,
}

impl raw::channel::notification::Notify for Notify {
    /// TODO/ documentation
	type Received = Received;

	type Error = Error;

    /// TODO/ documentation
    fn notify(mut self) -> Result<(), Error> {
        self.sender.try_send(())
    }
}

impl raw::channel::notification::Received for Received {}

impl Future for Received {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<bool> {
        Pin::new(&mut self.get_mut().recver).poll(ctx).map(|res| res.is_some())
    }
}
