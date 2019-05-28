#![feature(async_await)]

use std::task::Poll;

use aktoro_channel::*;
use futures_util::poll;
use futures_util::SinkExt;
use futures_util::StreamExt;

type Sender = unbounded::Sender<u8>;
type Receiver = unbounded::Receiver<u8>;

#[runtime::test]
async fn test() {
    // NORMAL
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);
    send_ok(24, &mut send);

    recv_ok(42, &mut recv);
    recv_ok(24, &mut recv);
    recv_empty(&mut recv);

    // SINK
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    assert_eq!(SinkExt::send(&mut send, 42).await, Ok(()));
    assert_eq!(SinkExt::send(&mut send, 24).await, Ok(()));

    recv_ok(42, &mut recv);
    recv_ok(24, &mut recv);
    recv_empty(&mut recv);

    // STREAM
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);
    send_ok(24, &mut send);

    assert_eq!(recv.next().await, Some(42));
    assert_eq!(recv.next().await, Some(24));
    assert_eq!(poll!(recv.next()), Poll::Pending);

    // DISCONNECTING SEND
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);

    {
        let mut send = send.clone();

        send_ok(24, &mut send);

        assert_eq!(send.disconnect(), Ok(()));
        assert!(!send.closed);
        assert!(send.disconnected);
        assert_eq!(send.disconnect(), Err(DisconnectError::Disconnected));

        assert_eq!(send.close(), Err(CloseError::Disconnected));

        send_disconnected(12, &mut send);

        assert_eq!(
            SinkExt::send(&mut send, 32).await,
            Err(SendError::Disconnected(()))
        );
    }

    send_is_default(&send);

    recv_ok(42, &mut recv);
    recv_ok(24, &mut recv);

    send_ok(32, &mut send);
    recv_ok(32, &mut recv);

    recv_empty(&mut recv);

    // CLOSING SEND BY DISCONNECTION
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);

    assert_eq!(send.disconnect(), Ok(()));
    assert!(!send.closed);
    assert!(send.disconnected);
    assert_eq!(send.disconnect(), Err(DisconnectError::Disconnected));

    assert_eq!(send.close(), Err(CloseError::Disconnected));

    recv_ok(42, &mut recv);

    send_disconnected(24, &mut send);
    recv_closed(&mut recv);

    assert_eq!(
        SinkExt::send(&mut send, 32).await,
        Err(SendError::Disconnected(()))
    );

    // CLOSING SEND
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);

    {
        let mut send = send.clone();

        assert_eq!(send.close(), Ok(()));
        assert!(send.closed);
        assert!(!send.disconnected);
        assert_eq!(send.close(), Err(CloseError::Closed));

        assert_eq!(send.disconnect(), Err(DisconnectError::Closed));

        send_closed(24, &mut send);
    }

    assert_eq!(send.close(), Err(CloseError::Closed));
    assert!(send.closed);
    assert!(!send.disconnected);

    assert_eq!(send.disconnect(), Err(DisconnectError::Closed));
    // FIXME: assert_eq!(recv.close(), Err(CloseError::Closed));
    assert_eq!(recv.close(), Ok(()));
    assert!(recv.closed);

    recv_ok(42, &mut recv);

    send_closed(24, &mut send);
    recv_closed(&mut recv);

    assert_eq!(
        SinkExt::send(&mut send, 32).await,
        Err(SendError::Closed(()))
    );

    // CLOSING RECV
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);

    assert_eq!(recv.close(), Ok(()));
    assert!(recv.closed);
    assert_eq!(recv.close(), Err(CloseError::Closed));

    assert_eq!(send.disconnect(), Err(DisconnectError::Closed));
    assert_eq!(send.close(), Err(CloseError::Closed));
    assert!(send.closed);
    assert!(!send.disconnected);

    recv_ok(42, &mut recv);

    send_closed(24, &mut send);
    recv_closed(&mut recv);

    assert_eq!(recv.next().await, None);

    // DROPING SEND
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);

    drop(send);

    recv_ok(42, &mut recv);
    recv_closed(&mut recv);
    assert!(recv.closed);

    assert_eq!(recv.close(), Err(CloseError::Closed));
    assert!(recv.closed);

    // DROPING RECV
    let (mut send, mut recv) = unbounded::new::<u8>();

    send_is_default(&send);
    recv_is_default(&mut recv);

    send_ok(42, &mut send);

    drop(recv);

    send_closed(24, &mut send);
    assert!(send.closed);

    assert_eq!(send.disconnect(), Err(DisconnectError::Closed));
    assert_eq!(send.close(), Err(CloseError::Closed));
    assert!(send.closed);
    assert!(!send.disconnected);
}

fn send_is_default(send: &Sender) {
    assert!(!send.closed);
    assert!(!send.disconnected);
}

fn recv_is_default(recv: &mut Receiver) {
    assert!(!recv.closed);
    recv_empty(recv);
}

fn send_ok(data: u8, send: &mut Sender) {
    assert_eq!(send.send(data), Ok(()));
    assert!(!send.closed);
    assert!(!send.disconnected);
}

fn send_disconnected(data: u8, send: &mut Sender) {
    assert_eq!(send.send(data), Err(SendError::Disconnected(data)));
    assert!(!send.closed);
    assert!(send.disconnected);
}

fn send_closed(data: u8, send: &mut Sender) {
    assert_eq!(send.send(data), Err(SendError::Closed(data)));
    assert!(send.closed);
    assert!(!send.disconnected);
}

fn recv_ok(data: u8, recv: &mut Receiver) {
    assert_eq!(recv.try_recv(), Ok(data));
}

fn recv_empty(recv: &mut Receiver) {
    assert_eq!(recv.try_recv(), Err(ReceiveError::Empty));
    assert!(!recv.closed);
}

fn recv_closed(recv: &mut Receiver) {
    assert_eq!(recv.try_recv(), Err(ReceiveError::Closed));
    assert!(recv.closed);
}
