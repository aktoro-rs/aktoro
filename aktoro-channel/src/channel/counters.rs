use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// TODO: documentation
pub(crate) struct Counters {
    /// TODO: documentation
    cmsgs: Option<AtomicUsize>,
    /// TODO: documentation
    csenders: AtomicUsize,
    /// TODO: documentation
    crecvers: AtomicUsize,

    /// TODO: documentation
    lmsgs: Option<usize>,
    /// TODO: documentation
    lsenders: Option<usize>,
    /// TODO: documentation
    lrecvers: Option<usize>,
}

impl Counters {
    /// TODO: documentation
    pub(crate) fn new(msgs: Option<usize>, senders: Option<usize>, recvers: Option<usize>) -> Counters {
        Counters {
            cmsgs: msgs.map(|_| AtomicUsize::new(0)),
            csenders: AtomicUsize::new(0),
            crecvers: AtomicUsize::new(0),

            lmsgs: msgs,
            lsenders: senders,
            lrecvers: recvers,
        }
    }

    /// TODO: documentation
    pub(crate) fn senders(&self) -> usize {
        self.csenders.load(Ordering::SeqCst)
    }

    /// TODO: documentation
    pub(crate) fn add_msg(&self) -> Result<(), ()> {
        if let Some(counter) = &self.cmsgs {
            let limit = self.lmsgs.unwrap();

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

    /// TODO: documentation
    pub(crate) fn add_sender(&self) -> Result<(), ()> {
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

    /// TODO: documentation
    pub(crate) fn add_recver(&self) -> Result<(), ()> {
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

    /// TODO: documentation
    pub(crate) fn sub_sender(&self) -> usize {
        loop {
            let cur = self.csenders.load(Ordering::SeqCst);
            if cur == 0 {
                return cur;
            }

            if self
                .csenders
                .compare_and_swap(cur, cur - 1, Ordering::SeqCst)
                == cur
            {
                return cur - 1;
            }
        }
    }

    /// TODO: documentation
    pub(crate) fn sub_recver(&self) -> usize {
        loop {
            let cur = self.crecvers.load(Ordering::SeqCst);
            if cur == 0 {
                return cur;
            }

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
