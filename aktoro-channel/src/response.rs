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
pub(crate) fn new<R>() -> (Respond<R>, Response<R>) {
    let (sender, recver) = once::new();
    (Respond { sender, }, Response { recver, })
}

/// TODO/ documentation
pub struct Respond<R> {
    /// TODO/ documentation
    sender: Sender<R>,
}

/// TODO/ documentation
pub struct Response<R> {
    /// TODO/ documentation
    recver: Receiver<R>,
}

impl<R> raw::channel::response::Respond<R> for Respond<R> {
    /// TODO/ documentation
	type Response = Response<R>;

	type Error = Error<R>;

    /// TODO/ documentation
    fn send(mut self, response: R) -> Result<(), Error<R>> {
        self.sender.try_send(response)
    }
}

impl<R> raw::channel::response::Response<R> for Response<R> {}

impl<R> Future for Response<R> {
    type Output = Option<R>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<R>> {
        Pin::new(&mut self.get_mut().recver).poll(ctx)
    }
}
