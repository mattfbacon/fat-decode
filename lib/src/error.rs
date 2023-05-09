#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Error {
	NotFat32,
	NotDirectory,
	NotFound,
}

impl std::fmt::Display for Error {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotFat32 => "not fat32",
			Self::NotDirectory => "not a directory",
			Self::NotFound => "not found",
		}
		.fmt(formatter)
	}
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = core::result::Result<T, E>;
