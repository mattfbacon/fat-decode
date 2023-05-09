#![deny(
	absolute_paths_not_starting_with_crate,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod dir;
mod error;
mod file;
mod info;
mod int_ext;
mod iter_ext;
mod long_name;
mod raw_file;
mod read;

pub use crate::dir::{Dir, Entry, EntryType};
pub use crate::error::{Error, Result};
pub use crate::file::File;
use crate::info::Info;
pub use crate::raw_file::RawFile;
pub use crate::read::Reader;
use crate::read::ReaderExt as _;

const FAT_ENTRY_SIZE: u32 = 4;
const FAT_ENTRY_MASK: u32 = 0x0fff_ffff;
const DIR_ENTRY_SIZE: usize = 32;
const FIRST_DATA_CLUSTER: u32 = 2;
const END_OF_CHAIN: u32 = 0xfff_fff8;

const ATTR_READ_ONLY: u8 = 1 << 0;
const ATTR_HIDDEN: u8 = 1 << 1;
const ATTR_SYSTEM: u8 = 1 << 2;
const ATTR_VOLUME_ID: u8 = 1 << 3;
const ATTR_DIRECTORY: u8 = 1 << 4;
// const ATTR_DIRTY_SINCE_LAST_BACKUP: u8 = 1 << 5;
const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;
const ATTR_RESERVED: u8 = 0b1100_0000;
const ATTR_MASK: u8 = !ATTR_RESERVED;

#[derive(Debug, Clone)]
pub struct Fat<R> {
	inner: R,
	info: Info,
}

impl<R: Reader> Fat<R> {
	pub fn new(inner: R) -> Result<Self> {
		let info = Info::read(&inner)?;
		Ok(Self { inner, info })
	}

	pub fn root(&self) -> Dir<'_, R> {
		Dir::from_raw(self.file_at_cluster(self.info.root_cluster))
	}

	fn file_at_cluster(&self, cluster: u32) -> RawFile<'_, R> {
		RawFile::at_cluster(self, cluster)
	}

	pub fn find_raw(&self, path: &str) -> Result<Entry<'_, R>> {
		let path = path.strip_prefix('/').unwrap_or(path);
		let components = path.split('/').filter(|component| !component.is_empty());

		let mut entry = Entry {
			fat: self,
			type_: EntryType::Directory,
			name: [0u8; 256],
			size: 0,
			first_cluster: self.info.root_cluster,
		};

		'components: for component in components {
			for child in entry.read_dir()? {
				let child = child?;

				if child.name() == component.as_bytes() {
					entry = child;
					continue 'components;
				}
			}

			return Err(Error::NotFound);
		}

		Ok(entry)
	}

	pub fn read_dir(&self, path: &str) -> Result<Dir<'_, R>> {
		self.find_raw(path)?.read_dir()
	}

	pub fn open(&self, path: &str) -> Result<File<'_, R>> {
		self.find_raw(path)?.open()
	}

	fn next_cluster(&self, cluster: u32) -> Result<u32> {
		let offset = self.info.cluster_to_fat(cluster);
		let next: u32 = self.inner.read_at(offset)?;
		let next = next & FAT_ENTRY_MASK;
		Ok(next)
	}
}
