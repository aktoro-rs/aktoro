use futures_core::future::BoxFuture;

pub trait Controller {
    type Action;
    type Status;

    fn send(&mut self, action: Self::Action) -> BoxFuture<Self::Status>;
}

pub trait Controlled {
    type Controller: Controller;

    fn try_recv(&mut self) -> Option<(
        <Self::Controller as Controller>::Action,
        <Self::Controller as Controller>::Status
    )>;
}
