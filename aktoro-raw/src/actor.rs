use std::error::Error as StdError;

use crate::context::Context;

pub trait Actor: Unpin + Send + Sized + 'static {
    type Context: Context<Self>;

    type Status: Status + Unpin;

    type Error: StdError + Send;

    #[allow(unused)]
    /// Called when the actor's context has been created
    /// but it hasn't been spawned by the runtime yet.
    fn starting(&mut self, ctx: &mut Self::Context) {}

    #[allow(unused)]
    /// Called when the actor's has been spawned by the
    /// runtime and polled for the first time.
    fn started(&mut self, ctx: &mut Self::Context) {}

    #[allow(unused)]
    /// Called when the actor has been asked to stop
    /// but has been left the option to cancel it.
    fn stopping(&mut self, ctx: &mut Self::Context) {}

    #[allow(unused)]
    /// Called when the actor has accepted to stop or
    /// it has been asked to stop immediately.
    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

pub trait Status: PartialEq + Default + Clone + Send {
    /// Returns the status that an actor should have
    /// before [`Actor::starting`] is called.
    ///
    /// [`Actor::starting`]: trait.Actor.html#method.starting
    fn starting() -> Self;

    /// Returns the status that an actor should have
    /// before [`Actor::started`] is called.
    ///
    /// [`Actor::started`]: trait.Actor.html#methood.started
    fn started() -> Self;

    /// Returns the status that an actor should have
    /// before [`Actor::stopping`] is called.
    ///
    /// ## Note
    ///
    /// If after [`Actor::stopping`] is called, its
    /// status is still the same it will be stopped.
    ///
    /// [`Actor::stopping`]: trait.Actor.html#method.stopping
    fn stopping() -> Self;

    /// Returns the status that an actor should have
    /// before [`Actor::stopped`] is called.
    ///
    /// [`Actor::stopped`]: trait.Actor.html#method.stopped
    fn stopped() -> Self;

    /// Returns the status that an actor will have
    /// after [`Actor::stopped`] has been called.
    ///
    /// [`Actor::stopped`]: trait.Actor.html#method.stopped
    fn dead() -> Self;

    fn is_starting(&self) -> bool;
    fn is_started(&self) -> bool;
    fn is_stopping(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn is_dead(&self) -> bool;
}
