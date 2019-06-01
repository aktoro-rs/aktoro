use crate::notify::Notify;

pub(crate) struct Message<T> {
    pub(crate) msg: T,
    notify: Option<Notify>,
}

impl<T> Message<T> {
    pub(crate) fn new(msg: T) -> Self {
        Message { msg, notify: None }
    }

    pub(crate) fn new_notified(msg: T) -> (Self, Notify) {
        let notify = Notify::new();

        (
            Message {
                msg,
                notify: Some(notify.clone()),
            },
            notify,
        )
    }

    pub(crate) fn unwrap(self) -> T {
        if let Some(notify) = self.notify {
            notify.done();
        }

        self.msg
    }
}
