use crate::notify::Notify;

/// A wrapper around a message that
/// a sender sent and an optional
/// notifier.
pub(crate) struct Message<T> {
    pub(crate) msg: T,
    notify: Option<Notify>,
}

impl<T> Message<T> {
    /// Creates a new `Message`
    /// contaning `msg` but no
    /// notifier.
    pub(crate) fn new(msg: T) -> Self {
        Message { msg, notify: None }
    }

    /// Creates a new `Message`
    /// containing `msg`, creating
    /// a new notifier and returning
    /// it.
    pub(crate) fn new_notified(msg: T) -> (Self, Notify) {
        let notify = Notify::new();

        (
            Message {
                msg,
                notify: Some(notify.0),
            },
            notify.1,
        )
    }

    /// Returns the inner `msg`,
    /// consuming the `Message`.
    pub(crate) fn unwrap(self) -> T {
        if let Some(notify) = self.notify {
            notify.done();
        }

        self.msg
    }
}
