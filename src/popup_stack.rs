use crate::queue::StackablePopupOpen;

#[derive(Default)]
pub struct PopupStack {
	stack: Vec<StackablePopupOpen>,
}

impl PopupStack {
	pub fn push(&mut self, popup: StackablePopupOpen) {
		self.stack.push(popup);
	}

	pub fn pop(&mut self) -> Option<StackablePopupOpen> {
		self.stack.pop()
	}

	pub fn clear(&mut self) {
		self.stack.clear();
	}

	#[cfg(test)]
	pub const fn is_empty(&self) -> bool {
		self.stack.is_empty()
	}
}
