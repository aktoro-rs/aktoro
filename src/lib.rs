pub use aktoro_raw as raw;

#[cfg(feature = "context")]
pub use aktoro_context as context;

#[cfg(feature = "runtime")]
pub use aktoro_runtime as runtime;

pub mod prelude {
    pub use aktoro_raw::Actor;
    pub use aktoro_raw::Context as RawContext;

    pub use aktoro_raw::ActionHandler;
    pub use aktoro_raw::EventHandler;
    pub use aktoro_raw::Handler;

    pub use aktoro_raw::Runtime as RawRuntime;
    pub use aktoro_raw::Spawned;

    pub use aktoro_raw::Controlled as RawControlled;
    pub use aktoro_raw::Controller as RawController;
    pub use aktoro_raw::Receiver as RawReceiver;
    pub use aktoro_raw::Sender as RawSender;
    pub use aktoro_raw::Updated as RawUpdated;
    pub use aktoro_raw::Updater as RawUpdater;

    pub use aktoro_raw::TcpClient as RawTcpClient;
    pub use aktoro_raw::TcpServer as RawTcpServer;
    pub use aktoro_raw::TcpStream as RawTcpStream;
    pub use aktoro_raw::UdpSocket as RawUdpSocket;

    #[cfg(feature = "context")]
    pub use crate::context::*;

    #[cfg(feature = "runtime")]
    pub use crate::runtime::*;
}
