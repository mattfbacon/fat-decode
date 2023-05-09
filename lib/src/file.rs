use crate::error::Result;
use crate::raw_file::RawFile;
use crate::read::Reader;

#[derive(Debug)]
pub struct File<'a, R> {
	raw: RawFile<'a, R>,
	remaining: u32,
}

impl<'a, R: Reader> File<'a, R> {
	#[must_use]
	pub fn from_raw(raw: RawFile<'a, R>, size: u32) -> Self {
		Self {
			raw,
			remaining: size,
		}
	}

	pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let len = buf.len().min(self.remaining as usize);
		let num_read = self.raw.read(&mut buf[..len])?;
		self.remaining -= u32::try_from(num_read).unwrap_or_else(|_| unreachable!());
		Ok(num_read)
	}
}
