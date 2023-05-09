pub trait IteratorExt {
	type Item;
	fn collect_array<const N: usize>(self) -> [Self::Item; N];
	fn collect_stack_buffer(self, buf: &mut [Self::Item]) -> &mut [Self::Item];
}

impl<I: Iterator> IteratorExt for I {
	type Item = <I as Iterator>::Item;

	fn collect_array<const N: usize>(mut self) -> [Self::Item; N] {
		let ret = std::array::from_fn(|_| self.next().unwrap());
		assert!(self.next().is_none());
		ret
	}

	fn collect_stack_buffer(self, buf: &mut [Self::Item]) -> &mut [Self::Item] {
		let mut i = 0;
		for item in self {
			buf[i] = item;
			i += 1;
		}
		&mut buf[..i]
	}
}

pub fn u8_to_u16(mut iter: impl Iterator<Item = u8>) -> impl Iterator<Item = u16> {
	std::iter::from_fn(move || {
		let a = iter.next()?;
		let b = iter.next()?;
		Some(u16::from_le_bytes([a, b]))
	})
}
