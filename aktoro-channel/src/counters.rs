use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Counters used to store the number
/// of senders, receivers, messages
/// sent and the channel's limits.
pub(crate) struct Counters {
    /// The number of messages sent
    /// over the channel.
    cmsgs: Option<AtomicUsize>,
    /// The number of senders
    /// connected to the channel.
    csenders: AtomicUsize,
    /// The number of receivers
    /// connected to the channel.
    crecvers: AtomicUsize,

    /// The number of messages that
    /// can be sent over the channel
    /// (in total).
    lmsgs: Option<usize>,
    /// The maximum number of senders
    /// that can be connected to the
    /// channel.
    lsenders: Option<usize>,
    /// The maximum number of
    /// receivers that can be connected
    /// to the channel.
    lrecvers: Option<usize>,
}

impl Counters {
    /// Creates counters with the
    /// specified limits.
    pub(crate) fn new(msgs: Option<usize>, senders: Option<usize>, recvers: Option<usize>) -> Self {
        Counters {
            cmsgs: msgs.map(|_| AtomicUsize::new(0)),
            csenders: AtomicUsize::new(0),
            crecvers: AtomicUsize::new(0),

            lmsgs: msgs,
            lsenders: senders,
            lrecvers: recvers,
        }
    }

    /// Gets the current number of
    /// senders connected to the channel.
    pub(crate) fn senders(&self) -> usize {
        self.csenders.load(Ordering::SeqCst)
    }

    /// Increases the total number of
    /// messages sent over the channel if
    /// necessary.
    pub(crate) fn add_msg(&self) -> Result<(), ()> {
        if let Some(counter) = &self.cmsgs {
            let limit = self.lmsgs.unwrap();

            // We CAS the sent messages
            // counter to increase it of
            // 1 or return an error if it
            // is above the limit.
            loop {
                let cur = counter.load(Ordering::SeqCst);
                let new = cur + 1;

                if new > limit {
                    return Err(());
                }

                if counter.compare_and_swap(cur, new, Ordering::SeqCst) == cur {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Increases the counter for the
    /// number of senders connected to the
    /// channel.
    pub(crate) fn add_sender(&self) -> Result<(), ()> {
        // If there is a limit for the number
        // of senders connected to the channel,
        // we CAS the counter to increase it of
        // 1 and return an error if it is
        // above the limit.
        if let Some(limit) = &self.lsenders {
            loop {
                let cur = self.csenders.load(Ordering::SeqCst);
                let new = cur + 1;

                if new > *limit {
                    return Err(());
                }

                if self.csenders.compare_and_swap(cur, new, Ordering::SeqCst) == cur {
                    break;
                }
            }
        } else {
            self.csenders.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Increases the counter for the
    /// number of receivers connected to
    /// the channel.
    pub(crate) fn add_recver(&self) -> Result<(), ()> {
        // If there is a limit for the number
        // or receivers that can be connected
        // to the channel, we CAS the counter
        // to increase it of 1 and return an
        // error if it is above the limit.
        if let Some(limit) = &self.lrecvers {
            loop {
                let cur = self.crecvers.load(Ordering::SeqCst);
                let new = cur + 1;

                if new > *limit {
                    return Err(());
                }

                if self.crecvers.compare_and_swap(cur, new, Ordering::SeqCst) == cur {
                    break;
                }
            }
        } else {
            self.crecvers.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Decreases the number of senders,
    /// returning the updated number.
    pub(crate) fn sub_sender(&self) -> usize {
        loop {
            let cur = self.csenders.load(Ordering::SeqCst);

            if self
                .csenders
                .compare_and_swap(cur, cur - 1, Ordering::SeqCst)
                == cur
            {
                return cur - 1;
            }
        }
    }

    /// Decreases the number of receivers,
    /// returning the updated number.
    pub(crate) fn sub_recver(&self) -> usize {
        loop {
            let cur = self.crecvers.load(Ordering::SeqCst);

            if self
                .crecvers
                .compare_and_swap(cur, cur - 1, Ordering::SeqCst)
                == cur
            {
                return cur - 1;
            }
        }
    }
}
