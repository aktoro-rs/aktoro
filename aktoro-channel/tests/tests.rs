#![feature(async_await)]

use aktoro_channel::*;
use futures_util::poll;
use futures_util::StreamExt;

#[test]
fn base() {
    let (sender, recver) = Builder::new().build();
    works(&sender, &recver);
}

#[runtime::test]
async fn async_base() {
    let (sender, recver) = Builder::new().build();
    async_works(&sender, &recver).await;
}

#[runtime::test]
async fn notify() {
    let (sender, recver) = Builder::new().build();
    async_works(&sender, &recver).await;

    let mut notify = sender.try_send_notify(2).unwrap();
    assert!(poll!(&mut notify).is_pending());

    assert_eq!(recver.try_recv(), Ok(Some(2)));
    assert!(poll!(&mut notify).is_ready());
}

#[test]
fn disconnect_sender() {
    let (mut sender, recver) = Builder::new().build();
    works(&sender, &recver);

    assert_eq!(sender.try_send(2), Ok(()));

    sender.disconnect();

    assert!(sender.is_closed());
    assert!(recver.is_closed());

    assert_eq!(recver.try_recv(), Ok(Some(2)));
    assert!(recver.try_recv().unwrap_err().is_closed());
}

#[test]
fn disconnect_recver() {
    let (sender, mut recver) = Builder::new().build();
    works(&sender, &recver);

    recver.disconnect();

    assert!(sender.is_closed());
    assert!(recver.is_closed());
}

#[test]
fn close_sender() {
    let (sender, recver) = Builder::new().build();
    works(&sender, &recver);

    assert_eq!(sender.try_send(2), Ok(()));

    sender.close_channel();

    assert!(sender.is_closed());
    assert!(recver.is_closed());

    assert_eq!(recver.try_recv(), Ok(Some(2)));
    assert!(sender.try_send(1).unwrap_err().is_closed());
    assert!(recver.try_recv().unwrap_err().is_closed());
}

#[test]
fn close_recver() {
    let (sender, recver) = Builder::new().build();
    works(&sender, &recver);

    assert_eq!(sender.try_send(2), Ok(()));

    recver.close_channel();

    assert!(sender.is_closed());
    assert!(recver.is_closed());

    assert_eq!(recver.try_recv(), Ok(Some(2)));
    assert!(sender.try_send(1).unwrap_err().is_closed());
    assert!(recver.try_recv().unwrap_err().is_closed());
}

#[test]
fn drop_senders() {
    let (sender, recver) = Builder::new().build();
    works(&sender, &recver);

    assert_eq!(sender.try_send(2), Ok(()));

    drop(sender);

    assert!(recver.is_closed());
    assert_eq!(recver.try_recv(), Ok(Some(2)));
    assert!(recver.try_recv().unwrap_err().is_closed());
}

#[test]
fn drop_recvers() {
    let (sender, recver) = Builder::new().build();
    works(&sender, &recver);

    drop(recver);

    assert!(sender.is_closed());
    assert!(sender.try_send(2).unwrap_err().is_closed());
}

#[test]
fn msgs_limit() {
    let (sender, recver) = Builder::new().limited_msgs(3).build();
    works(&sender, &recver);

    assert_eq!(sender.try_send(2), Ok(()));
    assert!(sender.try_send(3).unwrap_err().is_limit());
}

#[test]
fn senders_limit() {
    let (sender, recver) = Builder::new().limited_senders(3).build();
    works(&sender, &recver);

    let _2 = sender.try_clone().unwrap();
    let _1 = sender.try_clone().unwrap();

    assert!(sender.try_clone().is_err());
}

#[test]
fn recvers_limit() {
    let (sender, recver) = Builder::new().limited_receivers(3).build();
    works(&sender, &recver);

    let _2 = recver.try_clone().unwrap();
    let _1 = recver.try_clone().unwrap();

    assert!(recver.try_clone().is_err());
}

fn works(sender: &Sender<i32>, recver: &Receiver<i32>) {
    let (sender1, recver1) = (sender.try_clone().unwrap(), recver.try_clone().unwrap());

    let (sender2, recver2) = (sender1.try_clone().unwrap(), recver1.try_clone().unwrap());

    assert_eq!(recver1.try_recv(), Ok(None));
    assert_eq!(recver2.try_recv(), Ok(None));

    assert_eq!(sender1.try_send(0), Ok(()));
    assert_eq!(sender2.try_send(1), Ok(()));

    assert_eq!(recver2.try_recv(), Ok(Some(0)));
    assert_eq!(recver1.try_recv(), Ok(Some(1)));

    assert_eq!(recver1.try_recv(), Ok(None));
    assert_eq!(recver2.try_recv(), Ok(None));
}

async fn async_works<'c>(sender: &'c Sender<i32>, recver: &'c Receiver<i32>) {
    let (sender1, mut recver1) = (sender.try_clone().unwrap(), recver.try_clone().unwrap());

    let (sender2, mut recver2) = (sender1.try_clone().unwrap(), recver1.try_clone().unwrap());

    let mut next1 = recver1.next();
    let mut next2 = recver2.next();

    assert!(poll!(&mut next1).is_pending());
    assert!(poll!(&mut next2).is_pending());

    assert_eq!(sender1.try_send(0), Ok(()));
    assert_eq!(sender2.try_send(1), Ok(()));

    assert_eq!(next1.await, Some(0));
    assert_eq!(next2.await, Some(1));

    assert_eq!(recver1.try_recv(), Ok(None));
    assert_eq!(recver2.try_recv(), Ok(None));
}
