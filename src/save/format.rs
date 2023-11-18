use std::collections::HashMap;
use bevy::prelude::default;
use bincode::Options;
use serde::Deserialize;
use serde::Serialize;

use crate::networking::Username;
use crate::raw_id::RawId;
use crate::tile::TileData;
use crate::tile::WorldTile;
use crate::TilePos;
use crate::util;
use crate::world::WorldBan;

use super::error::SaveError;

pub const MAGIC: [u8; 4] = [0xB0, 0x7A, 0x21, 0xCA];
pub const SAVE_VERSION: u32 = 0x0;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WorldSave {
	pub tiles: HashMap<TilePos, WorldTile>,
	pub bans: HashMap<Username, WorldBan>,
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
struct FileOffset(u32);

impl std::ops::Add<u32> for FileOffset {
    type Output = FileOffset;

    fn add(self, rhs: u32) -> Self::Output {
        FileOffset(self.0 + rhs)
    }
}

impl std::ops::Add<usize> for FileOffset {
    type Output = FileOffset;

    fn add(self, rhs: usize) -> Self::Output {
        self + rhs as u32
    }
}

impl From<usize> for FileOffset {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Into<usize> for FileOffset {
    fn into(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct OffsetTable {
	tiles_offset: FileOffset,
	bans_offset: FileOffset,
}

impl WorldSave {
	pub fn serialize(&self) -> Result<Vec<u8>, SaveError> {
		let tiles: Vec<u8> = self.tiles
			.iter()
			.map(
				|(pos, tile)| {
					let mut vec: Vec<u8> = vec![];
					vec.extend_from_slice(&pos.0.to_le_bytes());
					vec.extend_from_slice(&pos.1.to_le_bytes());
					vec.extend_from_slice(&tile.0.0.to_le_bytes());
					vec.push(tile.1.0);
					vec
				}
			)
			.flatten()
			.collect();
		
		let bans: Vec<u8> = util::serialize(&self.bans)?;
		
		let tiles_offset: FileOffset = MAGIC.len().into();
		let bans_offset = tiles_offset + tiles.len();
		let offset_table = OffsetTable {
			tiles_offset,
			bans_offset,
		};
		
		let mut vec: Vec<u8> = vec![];
		vec.extend_from_slice(&MAGIC);
		vec.extend_from_slice(&SAVE_VERSION.to_le_bytes());
		vec.extend(util::serialize(&offset_table)?);
		vec.extend(tiles);
		Ok(vec)
	}
	
	pub fn deserialize(vec: Vec<u8>) -> Result<Self, SaveError> {
		let mut header = vec
			.chunks_exact(4);
		
		let magic_matches = header
			.nth(0)
			.unwrap_or_default()
			.eq(&MAGIC);
		if !magic_matches {
			return Err(SaveError::MagicMissing(u32::from_le_bytes(MAGIC)))
		}
		
		let mut version: [u8; 4] = default();
		version.copy_from_slice(header.nth(0).unwrap_or_default());
		let version = u32::from_le_bytes(version);
		
		match version {
			0 => {
				let offset_table_size = util::OPTIONS_LE.serialized_size::<OffsetTable>(&default())? as usize;
				let offset_table: OffsetTable = util::deserialize(&vec[8..offset_table_size])?;
				
				let bans: HashMap<Username, WorldBan> = util::deserialize(&vec[offset_table.bans_offset.into()..offset_table.tiles_offset.into()])?;
				
				let tiles = vec[offset_table.tiles_offset.into()..]
					.to_vec()
					.into_iter()
					.collect::<Vec<u8>>()
					.chunks_exact(13)
					.map(
						|chunk| {
							let mut x: [u8; 4] = Default::default();
							x.copy_from_slice(&chunk[0..4]);
							let mut y: [u8; 4] = Default::default();
							y.copy_from_slice(&chunk[4..8]);
							let mut raw_id: [u8; 4] = Default::default();
							raw_id.copy_from_slice(&chunk[8..12]);
							
							(TilePos(i32::from_le_bytes(x), i32::from_le_bytes(y)), WorldTile(RawId(u32::from_le_bytes(raw_id)), TileData(chunk[13])))
						}
					)
					.collect::<HashMap<_, _>>();
				
				Ok(
					Self {
						tiles,
						bans,
					}
				)
			},
			_ => {
				Err(SaveError::InvalidVersion(version, SAVE_VERSION))
			},
		}
	}
}
