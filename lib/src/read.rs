use crate::error::Result;

pub trait Reader {
	fn read_exact_at(&self, offset: u64, buf: &mut [u8]) -> Result<()>;
}

impl<R: Reader + ?Sized> Reader for &R {
	fn read_exact_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
		R::read_exact_at(self, offset, buf)
	}
}

// XXX this should really not be a generic parameter, but rather an associated constant.
// But we can't say `[u8; Self::SIZE]`.
pub trait Readable<const SIZE: usize> {
	fn from_bytes(bytes: [u8; SIZE]) -> Self;
}

impl Readable<1> for u8 {
	fn from_bytes(bytes: [u8; 1]) -> Self {
		Self::from_le_bytes(bytes)
	}
}

impl Readable<2> for u16 {
	fn from_bytes(bytes: [u8; 2]) -> Self {
		Self::from_le_bytes(bytes)
	}
}

impl Readable<4> for u32 {
	fn from_bytes(bytes: [u8; 4]) -> Self {
		Self::from_le_bytes(bytes)
	}
}

pub trait ReaderExt {
	fn read_at<const SIZE: usize, T: Readable<SIZE>>(&self, offset: u64) -> Result<T>;
}

impl<R: Reader + ?Sized> ReaderExt for R {
	fn read_at<const SIZE: usize, T: Readable<SIZE>>(&self, offset: u64) -> Result<T> {
		let mut buf = [0u8; SIZE];
		self.read_exact_at(offset, &mut buf)?;
		Ok(T::from_bytes(buf))
	}
}
