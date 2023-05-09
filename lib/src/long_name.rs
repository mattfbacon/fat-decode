#[allow(clippy::module_name_repetitions /* easier to import */)]
pub struct LongNameBuf {
	buf: [u8; 256],
	remaining: usize,
}

impl LongNameBuf {
	pub fn new() -> Self {
		Self {
			buf: [0u8; 256],
			remaining: 256,
		}
	}

	pub fn push_component(&mut self, component: &[u8]) {
		let old_remaining = self.remaining;
		let new_remaining = self.remaining.checked_sub(component.len()).unwrap();
		self.remaining = new_remaining;
		self.buf[new_remaining..old_remaining].copy_from_slice(component);
	}

	pub fn data(&self) -> &[u8] {
		&self.buf[self.remaining..]
	}

	pub fn clear(&mut self) {
		self.remaining = 256;
	}

	pub fn len(&self) -> usize {
		256 - self.remaining
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}
