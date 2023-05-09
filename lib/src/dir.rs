use crate::error::{Error, Result};
use crate::file::File;
use crate::iter_ext::IteratorExt as _;
use crate::long_name::LongNameBuf;
use crate::raw_file::RawFile;
use crate::read::Reader;
use crate::{Fat, ATTR_DIRECTORY, ATTR_LONG_NAME, ATTR_MASK, ATTR_VOLUME_ID, DIR_ENTRY_SIZE};

const ENTRY_LAST: u8 = 0x00;
const ENTRY_FREE: u8 = 0xe5;

fn make_short_name(buf: &[u8]) -> [u8; 256] {
	let (name, extension) = buf.split_at(8);
	let name_len = name
		.iter()
		.rposition(|&b| b != b' ')
		.map_or(0, |last| last + 1);
	let ext_len = extension
		.iter()
		.rposition(|&b| b != b' ')
		.map_or(0, |last| last + 1);

	let mut ret = [0u8; 256];

	ret[..name_len].copy_from_slice(&name[..name_len]);
	if ext_len != 0 {
		ret[name_len] = b'.';
		ret[name_len + 1..][..ext_len].copy_from_slice(&extension[..ext_len]);
	}

	ret
}

#[derive(Debug)]
pub struct Dir<'a, R> {
	raw: RawFile<'a, R>,
	fused: bool,
}

impl<'a, R> Dir<'a, R> {
	pub(crate) fn from_raw(raw: RawFile<'a, R>) -> Self {
		Self { raw, fused: false }
	}
}

impl<'a, R: Reader> Iterator for Dir<'a, R> {
	type Item = Result<Entry<'a, R>>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.fused {
			return None;
		}

		let mut long_name_buf = LongNameBuf::new();
		loop {
			let mut buf = [0u8; DIR_ENTRY_SIZE];

			let num_read = match self.raw.read(&mut buf) {
				Ok(num_read) => num_read,
				Err(error) => return Some(Err(error)),
			};

			if num_read < buf.len() {
				self.fused = true;
				return None;
			}

			match buf[0] {
				ENTRY_LAST => {
					self.fused = true;
					return None;
				}
				ENTRY_FREE => {
					continue;
				}
				_ => {}
			}

			let raw_attr = buf[11];
			let attr = raw_attr & ATTR_MASK;

			if attr == ATTR_LONG_NAME {
				let decode_name = |start: usize, len_u16: usize| {
					let bytes = buf[start..][..len_u16 * 2].iter().copied();
					char::decode_utf16(crate::iter_ext::u8_to_u16(bytes)).map(Result::unwrap)
				};
				let name1 = decode_name(1, 5);
				let name2 = decode_name(14, 6);
				let name3 = decode_name(28, 2);
				let mut name_buf = [0u8; 32];
				let name: &mut [u8] = name1
					.chain(name2)
					.chain(name3)
					.take_while(|&b| b != '\0')
					.flat_map(|ch| {
						let mut buf = [0u8; 4];
						let len = ch.encode_utf8(&mut buf).len();
						buf.into_iter().take(len)
					})
					.collect_stack_buffer(&mut name_buf);
				long_name_buf.push_component(name);
				continue;
			}

			let size = u32::from_le_bytes(buf[28..32].try_into().unwrap());

			let type_ = if attr & ATTR_VOLUME_ID > 0 {
				EntryType::VolumeLabel
			} else if attr & ATTR_DIRECTORY > 0 {
				EntryType::Directory
			} else {
				EntryType::File
			};

			let short_name = &buf[0..11];
			let name = if long_name_buf.is_empty() {
				make_short_name(short_name)
			} else {
				let mut raw = long_name_buf.data().iter().copied();
				std::array::from_fn(|_| raw.next().unwrap_or(0))
			};

			long_name_buf.clear();

			let first_cluster = {
				let hi = u16::from_le_bytes(buf[20..22].try_into().unwrap());
				let lo = u16::from_le_bytes(buf[26..28].try_into().unwrap());
				u32::from(hi) << 16 | u32::from(lo)
			};

			break Some(Ok(Entry {
				fat: self.raw.fat,
				type_,
				size,
				name,
				first_cluster,
			}));
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
	File,
	Directory,
	VolumeLabel,
}

#[derive(Debug)]
pub struct Entry<'a, R> {
	pub(crate) fat: &'a Fat<R>,
	pub(crate) type_: EntryType,
	pub(crate) size: u32,
	pub(crate) name: [u8; 256],
	pub(crate) first_cluster: u32,
}

impl<'a, R: Reader> Entry<'a, R> {
	#[must_use]
	pub fn type_(&self) -> EntryType {
		self.type_
	}

	#[must_use]
	pub fn name(&self) -> &[u8] {
		let nul = self
			.name
			.iter()
			.copied()
			.position(|b| b == b'\0')
			.unwrap_or(self.name.len());

		&self.name[..nul]
	}

	#[must_use]
	pub fn size(&self) -> u32 {
		self.size
	}

	#[must_use]
	pub fn as_raw(&self) -> RawFile<'a, R> {
		self.fat.file_at_cluster(self.first_cluster)
	}

	pub fn read_dir(&self) -> Result<Dir<'a, R>> {
		if let EntryType::Directory = self.type_ {
			let raw = self.fat.file_at_cluster(self.first_cluster);
			Ok(Dir::from_raw(raw))
		} else {
			Err(Error::NotDirectory)
		}
	}

	pub fn open(&self) -> Result<File<'a, R>> {
		if let EntryType::File = self.type_ {
			let raw = self.fat.file_at_cluster(self.first_cluster);
			Ok(File::from_raw(raw, self.size))
		} else {
			Err(Error::NotDirectory)
		}
	}
}
