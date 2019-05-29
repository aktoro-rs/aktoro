use crate::context::Context;

pub trait Actor: Unpin + Send + Sized + 'static {
    type Context: Context<Self>;

    type Status: Status + Unpin;

    // TODO: starting
    // TODO: started
    // TODO: stopping
    // TODO: stopped
}

pub trait Status: Default + Send {} // TODO

impl Status for () {}
