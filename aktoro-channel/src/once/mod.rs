use std::sync::Arc;

/// TODO: module documentation

mod inner;
mod receiver;
mod sender;

use self::inner::Inner;

pub(super) use self::receiver::Receiver;
pub(super) use self::sender::Sender;

/// TODO: documentation
pub(crate) fn new<M>() -> (Sender<M>, Receiver<M>) {
    let inner = Arc::new(Inner::new());
    (Sender::new(inner.clone()), Receiver::new(inner))
}
