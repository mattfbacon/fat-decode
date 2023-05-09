use crate::error::Result;
use crate::read::Reader;
use crate::{Fat, END_OF_CHAIN};

#[derive(Debug)]
pub struct RawFile<'a, R> {
	pub(crate) fat: &'a Fat<R>,
	first_cluster: u32,
	current_cluster: u32,
	cursor_in_cluster: u32,
}

impl<'a, R: Reader> RawFile<'a, R> {
	pub fn at_cluster(fat: &'a Fat<R>, cluster: u32) -> RawFile<'_, R> {
		RawFile {
			fat,
			first_cluster: cluster,
			current_cluster: cluster,
			cursor_in_cluster: 0,
		}
	}

	/// Unlike `std::io::Read` this is guaranteed to fill the buffer if there are enough bytes left to do so. A partial read will only occur at EOF.
	pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let mut bytes_read = 0;

		while bytes_read < buf.len() && self.current_cluster < END_OF_CHAIN {
			let current_sector = self.fat.info.first_sector_of_cluster(self.current_cluster);
			let current_byte = u64::from(current_sector) * u64::from(self.fat.info.bytes_per_sector)
				+ u64::from(self.cursor_in_cluster);

			let remaining_in_cluster = self.fat.info.bytes_per_cluster() - self.cursor_in_cluster;
			let remaining_in_buf = buf.len() - bytes_read;

			let to_read = remaining_in_buf.min(
				remaining_in_cluster
					.try_into()
					.unwrap_or_else(|_| unreachable!()),
			);

			self
				.fat
				.inner
				.read_exact_at(current_byte, &mut buf[bytes_read..][..to_read])?;

			bytes_read += to_read;
			self.cursor_in_cluster += u32::try_from(to_read).unwrap_or_else(|_| unreachable!());

			if self.cursor_in_cluster == self.fat.info.bytes_per_cluster() {
				self.current_cluster = self.fat.next_cluster(self.current_cluster)?;
				self.cursor_in_cluster = 0;
			}
		}

		Ok(bytes_read)
	}

	pub fn rewind(&mut self) {
		self.current_cluster = self.first_cluster;
		self.cursor_in_cluster = 0;
	}
}
