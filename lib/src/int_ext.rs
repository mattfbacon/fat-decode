pub trait IntExt {
	fn div_ceil_p(self, divisor: Self) -> Self;
}

macro_rules! impl_int_ext {
	($($ty:ty),* $(,)?) => {
		$(impl IntExt for $ty {
			fn div_ceil_p(self, rhs: Self) -> Self {
				let quotient = self / rhs;
				let remainder = self % rhs;
				if remainder > 0 && rhs > 0 {
					quotient + 1
				} else {
					quotient
				}
			}
		})*
	};
}

impl_int_ext! { u8, u16, u32, u64, usize }
