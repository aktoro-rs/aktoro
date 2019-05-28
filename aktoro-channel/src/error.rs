#[derive(PartialEq, Eq, Debug)]
/// The error type that is returned by the channels'
/// senders when failing to send data.
pub enum SendError<D> {
    /// Returned when the channel's buffer is full.
    Full(D),
    /// Returned when the sender coudln't send the data
    /// because it previously disconnected itself from
    /// the channel.
    Disconnected(D),
    /// Returned when the channel has been closed.
    Closed(D),
}

#[derive(PartialEq, Eq, Debug)]
/// The error type that is returned by the channels'
/// receivers when failing to receive data.
pub enum ReceiveError {
    /// Returned when the channel's buffer is empty.
    Empty,
    /// Returned when the receiver coudln't receive data
    /// because it previously disconnected itself from
    /// the channel.
    Disconnected,
    /// Returned when the channel has been closed and
    /// its buffer is empty.
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
/// The error type that is returned by the channels'
/// senders and receivers when failing to disconnect
/// themselves from the channel.
pub enum DisconnectError {
    /// Returned when the sender/receiver already
    /// disconnected itself from the channel.
    Disconnected,
    /// Returned when the channel has already
    /// been closed.
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
/// The error type that is returned by the channels'
/// senders and receivers when failing to close the
/// channel.
pub enum CloseError {
    /// Returned when the sender/receiver already
    /// disconnected itself from the channel.
    Disconnected,
    /// Returned when the channel has already
    /// been closed.
    Closed,
}

impl<D> SendError<D> {
    /// Returns a reference to the data that the
    /// sender was trying to send over the channel.
    pub fn inner(&self) -> &D {
        match self {
            SendError::Full(data) => data,
            SendError::Disconnected(data) => data,
            SendError::Closed(data) => data,
        }
    }

    /// Returns a mutable reference to the data
    /// that the sender was trying to send over
    /// the channel.
    pub fn inner_mut(&mut self) -> &mut D {
        match self {
            SendError::Full(data) => data,
            SendError::Disconnected(data) => data,
            SendError::Closed(data) => data,
        }
    }

    /// Returns the data that the sender was trying
    /// to send over the channel.
    pub fn into_inner(self) -> D {
        match self {
            SendError::Full(data) => data,
            SendError::Disconnected(data) => data,
            SendError::Closed(data) => data,
        }
    }

    /// Maps a `SendError<D>` to a `SendError<E>` by
    /// applying a function to the inner value.
    pub fn map_inner<E, F>(self, op: F) -> SendError<E>
    where
        F: FnOnce(D) -> E,
    {
        match self {
            SendError::Full(data) => SendError::Full(op(data)),
            SendError::Disconnected(data) => SendError::Disconnected(op(data)),
            SendError::Closed(data) => SendError::Closed(op(data)),
        }
    }

    /// Whether the sender failed to send data
    /// because the channel's buffer was full.
    pub fn is_full(&self) -> bool {
        if let SendError::Full(_) = self {
            true
        } else {
            false
        }
    }

    /// Whether the sender failed to send data
    /// because it already disconnected itself from
    /// the channel.
    pub fn is_disconnected(&self) -> bool {
        if let SendError::Disconnected(_) = self {
            true
        } else {
            false
        }
    }

    /// Whether the sender failed to send data
    /// because the channel has been closed.
    pub fn is_closed(&self) -> bool {
        if let SendError::Closed(_) = self {
            true
        } else {
            false
        }
    }
}

impl ReceiveError {
    /// Whether the receiver failed to receive
    /// data because the channel's buffer is
    /// empty.
    pub fn is_empty(&self) -> bool {
        *self == ReceiveError::Empty
    }

    /// Whether the receiver failed to receive
    /// data because it already disconnected itself
    /// from the channel.
    pub fn is_disconnected(&self) -> bool {
        *self == ReceiveError::Disconnected
    }

    /// Whether the receiver failed to receive
    /// data because the channel has been closed
    /// and its buffer empty.
    pub fn is_closed(&self) -> bool {
        *self == ReceiveError::Closed
    }
}

impl DisconnectError {
    /// Whether the sender/receiver failed to
    /// disconnect itself from the channel because
    /// it was already disconnected.
    pub fn is_disconnected(&self) -> bool {
        *self == DisconnectError::Disconnected
    }

    /// Whether the sender/receiver failed to
    /// disconnect itself from the channel because
    /// the channel has been closed.
    pub fn is_closed(&self) -> bool {
        *self == DisconnectError::Closed
    }
}

impl CloseError {
    /// Whether the sender/receiver failed to
    /// close the channel because it already
    /// disconnected itself from it.
    pub fn is_disconnected(&self) -> bool {
        *self == CloseError::Disconnected
    }

    /// Whether the sender/receiver failed to
    /// close the channel because it was
    /// already closed.
    pub fn is_closed(&self) -> bool {
        *self == CloseError::Closed
    }
}
