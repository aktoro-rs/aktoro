use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub(crate) struct Counters {
    cmsgs: Option<AtomicUsize>,
    csenders: AtomicUsize,
    crecvers: AtomicUsize,

    lmsgs: Option<usize>,
    lsenders: Option<usize>,
    lrecvers: Option<usize>,
}

impl Counters {
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

    pub(crate) fn senders(&self) -> usize {
        self.csenders.load(Ordering::SeqCst)
    }

    pub(crate) fn add_msg(&self) -> Result<(), ()> {
        if let Some(counter) = &self.cmsgs {
            let limit = self.lmsgs.clone().unwrap();

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
