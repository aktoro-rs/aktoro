use super::Actor;

/// TODO: documentation
pub trait Status<A: Actor>: PartialEq + Unpin + Send {
    /// TODO: documentation
    fn starting() -> Self;

    /// TODO: documentation
    fn started() -> Self;

    /// TODO: documentation
    fn running() -> Self;

    /// TODO: documentation
    fn stopping() -> Self;

    /// TODO: documentation
    fn stopped() -> Self;

    /// TODO: documentation
    fn dead() -> Self;

    /// TODO: documentation
    fn panicked(error: A::Error) -> Self;

    /// TODO: documentation
    fn is_starting(&self) -> bool;
    /// TODO: documentation
    fn is_running(&self) -> bool;
    /// TODO: documentation
    fn is_started(&self) -> bool;
    /// TODO: documentation
    fn is_stopping(&self) -> bool;
    /// TODO: documentation
    fn is_stopped(&self) -> bool;
    /// TODO: documentation
    fn is_dead(&self) -> bool;
    /// TODO: documentation
    fn is_panicked(&self) -> bool;
}
