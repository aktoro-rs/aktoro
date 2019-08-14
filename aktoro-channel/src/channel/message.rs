use aktoro_raw as raw;
use aktoro_raw::channel::notification::Notify as RawNotify;
use aktoro_raw::channel::response::Respond as RawRespond;

use crate::error::Error;
use crate::notification;
use crate::notification::Notify;
use crate::notification::Received;
use crate::response;
use crate::response::Respond;
use crate::response::Response;

/// TODO: documentation
pub struct Message<I, O = ()> {
    inner: MessageInner<I, O>,
}

/// TODO: documentation
enum MessageInner<I, O = ()> {
    /// TODO: documentation
    Normal(I),
    /// TODO: documentation
    Notifying {
        msg: I,
        notify: Option<Notify>,
    },
    /// TODO: documentation
    Responding {
        msg: I,
        resp: Option<Respond<O>>,
    },
}

impl<I, O> Message<I, O> {
    /// TODO: documentation
    pub(crate) fn normal(msg: I) -> Message<I, O> {
        Message {
            inner: MessageInner::Normal(msg),
        }
    }

    /// TODO: documentation
    pub(crate) fn notifying(msg: I) -> (Message<I, O>, Received) {
        let (notify, received) = notification::new();
        (
            Message {
                inner: MessageInner::Notifying {
                    msg,
                    notify: Some(notify),
                },
            },
            received,
        )
    }

    /// TODO: documentation
    pub(crate) fn responding(msg: I) -> (Message<I, O>, Response<O>) {
        let (respond, response) = response::new();
        (
            Message {
				inner: MessageInner::Responding {
					msg,
					resp: Some(respond),
				},
			},
            response,
        )
    }
}

impl<I, O> raw::channel::Message<I, O> for Message<I, O> {
    /// TODO: documentation
	type Notify = Notify;

    /// TODO: documentation
	type Respond = Respond<O>;

    /// TODO: documentation
    fn msg(&self) -> &I {
        match &self.inner {
            MessageInner::Normal(msg) => msg,
            MessageInner::Notifying { msg, .. } => msg,
            MessageInner::Responding { msg, .. } => msg,
        }
    }

    /// TODO: documentation
    fn msg_mut(&mut self) -> &mut I {
        match &mut self.inner {
            MessageInner::Normal(msg) => msg,
            MessageInner::Notifying { msg, .. } => msg,
            MessageInner::Responding { msg, .. } => msg,
        }
    }

    /// TODO: documentation
    fn into_msg(self) -> I {
        match self.inner {
            MessageInner::Normal(msg) => msg,
            MessageInner::Notifying { msg, .. } => msg,
            MessageInner::Responding { msg, .. } => msg,
        }
    }

    /// TODO: documentation
    fn is_notifying(&self) -> bool {
        if let MessageInner::Notifying { .. } = self.inner {
            true
        } else {
            false
        }
    }

    /// TODO: documentation
    fn is_responding(&self) -> bool {
        if let MessageInner::Responding { .. } = self.inner {
            true
        } else {
            false
        }
    }

    /// TODO: documentation
    fn notify(&mut self) -> Result<(), Error> {
        if let MessageInner::Notifying { notify, .. } = &mut self.inner {
            if let Some(notify) = notify.take() {
                notify.notify()
            } else {
                Err(Error::closed(None))
            }
        } else {
            Err(Error::unsupported(None))
        }
    }

    /// TODO: documentation
	fn respond(&mut self, response: O) -> Result<(), Error<O>> {
		if let MessageInner::Responding { resp, .. } = &mut self.inner {
			if let Some(resp) = resp.take() {
				resp.send(response)
			} else {
				Err(Error::closed(Some(response)))
			}
		} else {
			Err(Error::unsupported(Some(response)))
		}
	}
}
