use crate::error::{Error, Result};
use crate::read::{Reader, ReaderExt as _};
use crate::{FAT_ENTRY_SIZE, FIRST_DATA_CLUSTER};

#[derive(Debug, Clone, Copy)]
pub struct Info {
	pub bytes_per_sector: u16,
	pub sectors_per_cluster: u8,
	pub reserved_sector_count: u16,
	pub first_data_sector: u32,
	pub root_cluster: u32,
}

impl Info {
	pub fn cluster_to_fat(&self, cluster: u32) -> u64 {
		let disk_offset = u64::from(cluster) * u64::from(FAT_ENTRY_SIZE);
		u64::from(self.reserved_sector_count) * u64::from(self.bytes_per_sector) + disk_offset
	}

	pub fn first_sector_of_cluster(&self, cluster: u32) -> u32 {
		((cluster - FIRST_DATA_CLUSTER) * u32::from(self.sectors_per_cluster)) + self.first_data_sector
	}

	pub fn bytes_per_cluster(&self) -> u32 {
		u32::from(self.sectors_per_cluster) * u32::from(self.bytes_per_sector)
	}

	pub fn read(reader: impl Reader) -> Result<Info> {
		// Directly read.
		let bytes_per_sector: u16 = reader.read_at(11)?;
		let sectors_per_cluster: u8 = reader.read_at(13)?;
		let reserved_sector_count: u16 = reader.read_at(14)?;
		let num_fats: u8 = reader.read_at(16)?;
		let root_entry_count: u16 = reader.read_at(17)?;
		let fat_size_16: u16 = reader.read_at(22)?;
		let fat_size_32: u32 = reader.read_at(36)?;
		let root_cluster: u32 = reader.read_at(44)?;

		// Calculated.
		if fat_size_16 != 0 {
			return Err(Error::NotFat32);
		}
		let fat_size = fat_size_32;

		if root_entry_count != 0 {
			return Err(Error::NotFat32);
		}

		let first_data_sector = u32::from(reserved_sector_count) + (u32::from(num_fats) * fat_size);

		let ret = Info {
			bytes_per_sector,
			sectors_per_cluster,
			reserved_sector_count,
			first_data_sector,
			root_cluster,
		};
		Ok(ret)
	}
}
