use std::future::Future;

/// TODO: documentation
pub trait Notify: Future<Output = ()> {
    /// TODO: documentation
    fn done(self);
}
