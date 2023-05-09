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

use std::cell::RefCell;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use fat_decode::Fat;

#[derive(Debug)]
struct Reader {
	inner: RefCell<BufReader<std::fs::File>>,
}

impl Reader {
	fn open(path: impl AsRef<Path>) -> Self {
		Self {
			inner: RefCell::new(BufReader::new(std::fs::File::open(path).unwrap())),
		}
	}
}

impl fat_decode::Reader for Reader {
	fn read_exact_at(&self, offset: u64, buf: &mut [u8]) -> fat_decode::Result<()> {
		let mut inner = self.inner.borrow_mut();
		inner.seek(SeekFrom::Start(offset)).unwrap();
		inner.read_exact(buf).unwrap();
		Ok(())
	}
}

struct IoAdapter<'a, 'b, R>(&'a mut fat_decode::File<'b, R>);

impl<R: fat_decode::Reader> Read for IoAdapter<'_, '_, R> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self
			.0
			.read(buf)
			.map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
	}
}

/// Decode FAT32 partitions.
#[derive(argh::FromArgs)]
struct Args {
	/// the path to the FAT partition to decode from
	#[argh(option, short = 'f')]
	fat: PathBuf,
	#[argh(subcommand)]
	subcommand: Subcommand,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Subcommand {
	Ls(LsArgs),
	Cat(CatArgs),
}

/// List a directory.
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "ls")]
struct LsArgs {
	#[argh(positional)]
	path: String,
}

/// Read a file.
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "cat")]
struct CatArgs {
	#[argh(positional)]
	path: String,
}

fn main() {
	let args: Args = argh::from_env();

	let reader = Reader::open(&args.fat);
	let fat = Fat::new(reader).unwrap();

	match args.subcommand {
		Subcommand::Ls(args) => {
			let dir = fat.read_dir(&args.path).unwrap();
			for entry in dir {
				let entry = entry.unwrap();
				println!(
					"{:?} {:?} {:?}",
					entry.type_(),
					std::str::from_utf8(entry.name()).unwrap(),
					entry.size()
				);
			}
		}
		Subcommand::Cat(args) => {
			let mut file = fat.open(&args.path).unwrap();
			std::io::copy(&mut IoAdapter(&mut file), &mut std::io::stdout()).unwrap();
		}
	}
}
