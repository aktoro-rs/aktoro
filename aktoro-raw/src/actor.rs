use crate::context::Context;

pub trait Actor: Unpin + Send + Sized + 'static {
    type Context: Context<Self>;

    type Status: Status + Unpin;

    type Error;

    #[allow(unused)]
    fn starting(&mut self, ctx: &mut Self::Context) {}

    #[allow(unused)]
    fn started(&mut self, ctx: &mut Self::Context) {}

    #[allow(unused)]
    fn stopping(&mut self, ctx: &mut Self::Context) {}

    #[allow(unused)]
    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

pub trait Status: Default + Send {
    fn is_stopping(&self) -> bool;
    fn is_stopped(&self) -> bool;
}
