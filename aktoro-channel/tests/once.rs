#![feature(async_await)]

use std::task::Poll;

use aktoro_channel::*;
use futures_util::poll;

type Sender = once::Sender<u8>;
type Receiver = once::Receiver<u8>;

#[runtime::test]
async fn test() {
    // NORMAL
    let (mut send, mut recv) = once::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);
    send_full(24, &mut send);

    recv_ok(42, &mut recv);
    recv_closed(&mut recv);

    // USING FUTURE
    let (mut send, mut recv) = once::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    assert_eq!(poll!(&mut recv), Poll::Pending);

    send_ok(42, &mut send);
    send_full(24, &mut send);

    assert_eq!(poll!(&mut recv), Poll::Ready(Ok(42)));
    assert!(recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    recv_closed(&mut recv);

    // CLOSING RECV
    let (mut send, mut recv) = once::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    assert_eq!(recv.close(), Ok(()));
    assert!(recv.closed);
    assert_eq!(recv.close(), Err(CloseError::Closed));

    send_closed(42, &mut send);
    recv_closed(&mut recv);

    // DROPING SEND
    let (send, mut recv) = once::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    drop(send);

    recv_closed(&mut recv);

    // DROPING RECV
    let (mut send, mut recv) = once::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    drop(recv);

    send_closed(42, &mut send);
}

fn send_is_default(send: &Sender) {
    assert!(!send.sent);
    assert!(!send.cancelled);
}

fn recv_is_default(recv: &mut Receiver) {
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);
    recv_empty(recv);
}

fn send_ok(data: u8, send: &mut Sender) {
    assert_eq!(send.send(data), Ok(()));
    assert!(send.sent);
    assert!(!send.cancelled);
}

fn send_full(data: u8, send: &mut Sender) {
    assert_eq!(send.send(data), Err(SendError::Full(data)));
    assert!(send.sent);
    assert!(!send.cancelled);
}

fn send_closed(data: u8, send: &mut Sender) {
    let sent = send.sent;
    assert_eq!(send.send(data), Err(SendError::Closed(data)));
    assert_eq!(send.sent, sent);
    assert!(send.cancelled);
}

fn recv_ok(data: u8, recv: &mut Receiver) {
    assert_eq!(recv.try_recv(), Ok(data));
    assert!(recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);
}

fn recv_empty(recv: &mut Receiver) {
    assert_eq!(recv.try_recv(), Err(ReceiveError::Empty));
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);
}

fn recv_closed(recv: &mut Receiver) {
    let received = recv.received;
    assert_eq!(recv.try_recv(), Err(ReceiveError::Closed));
    assert_eq!(recv.received, received);
    assert!(recv.closed || recv.cancelled);
}
