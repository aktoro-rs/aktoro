# aktoro-channel

Right now, this crate only provides wrappers around the channel types of
[`futures_channel`](https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/).

The long term goal is to write and use custom internals for its channels,
along with writing channels that are defined by 6 properties:
- whether it is uni- or bidirectional
- whether a sender gets notified when a message it has send, has been read
- the number of unread messages that it can hold
- the total number of messages that can be sent over it
- the number of sender that it can have
- the number of receiver that it can have
