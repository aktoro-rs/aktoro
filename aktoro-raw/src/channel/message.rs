use super::notification::Notify;
use super::response::Respond;

/// TODO: documentation
pub trait Message<I, O = ()> {
	/// TODO: documentation
    type Notify: Notify;

	/// TODO: documentation
	type Respond: Respond<O>;

	/// TODO: documentation
	fn msg(&self) -> &I;

	/// TODO: documentation
	fn msg_mut(&mut self) -> &mut I;

	/// TODO: documentation
	fn into_msg(self) -> I;

	/// TODO: documentation
	fn is_notifying(&self) -> bool;

	/// TODO: documentation
	fn is_responding(&self) -> bool;

	/// TODO: documentation
	fn notify(&mut self) -> Result<(), <Self::Notify as Notify>::Error>;

	/// TODO: documentation
	fn respond(&mut self, response: O) -> Result<(), <Self::Respond as Respond<O>>::Error>;
}
