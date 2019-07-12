#![feature(async_await)]

use std::pin::Pin;

use aktoro::prelude::*;
use aktoro::raw;
use futures_util::io::WriteHalf;
use futures_util::StreamExt;

mod agent;
mod client;
mod server;

use client::Client;
use server::Server;

struct Sent<S: raw::TcpStream>(Pin<Box<WriteHalf<S>>>);

struct Received(Vec<u8>);

#[runtime::main]
async fn main() {
    let mut rt = Runtime::new();
    let net = rt.net();

    let server = Server {
        tcp: Some(net.tcp_bind("127.0.0.1:5555").unwrap()),
        cancellable: None,
    };

    rt.spawn(server).unwrap();

    let client = Client::<TcpClient> {
        connect: Some(Box::pin(net.tcp_connect("127.0.0.1:5555").unwrap())),
        closed_connect: Some(Box::pin(net.tcp_connect("127.0.0.1:5555").unwrap())),
        write: None,
    };

    rt.spawn(client).unwrap();

    let mut wait = rt.wait();

    while let Some(res) = wait.next().await {
        res.expect("an error occured while waiting for the runtime to stop");
    }
}
