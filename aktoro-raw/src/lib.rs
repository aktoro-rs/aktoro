/// TODO: crate documentation
/// TODO(struct): actor supervisor
/// TODO(struct): actor schduler

mod context;

/// TODO: documentation
pub mod actor;

/// TODO: documentation
pub mod channel;

/// TODO: documentation
pub mod handler;

/// TODO: documentation
pub mod runtime;

pub use self::actor::Actor;
pub use self::channel::Receiver;
pub use self::channel::Sender;
pub use self::context::Context;
pub use self::handler::action;
pub use self::handler::event;
pub use self::handler::message;
pub use self::runtime::Runtime;
