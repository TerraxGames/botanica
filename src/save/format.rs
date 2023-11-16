use std::collections::HashMap;
use std::mem::size_of;
use serde::Deserialize;
use serde::Serialize;

use crate::raw_id::RawId;
use crate::tile::TileData;
use crate::tile::WorldTile;
use crate::TilePos;

use super::error::SaveReadError;

pub const MAGIC: [u8; 4] = [0xB0, 0x7A, 0x21, 0xCA];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorldSave {
	pub tiles: HashMap<TilePos, WorldTile>,
}

impl WorldSave {
	pub fn serialize(self) -> Vec<u8> {
		let tile_bytes: Vec<u8> = self.tiles
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
		
		let mut vec: Vec<u8> = vec![];
		vec.extend_from_slice(&MAGIC);
		vec.extend(tile_bytes);
		vec
	}
	
	pub fn deserialize(vec: Vec<u8>) -> Result<Self, SaveReadError> {
		let magic_matches = vec
			.chunks_exact(4)
			.nth(0)
			.unwrap_or_default()
			.eq(&MAGIC);
		if !magic_matches {
			return Err(SaveReadError::MagicMissing(u32::from_le_bytes(MAGIC)))
		}

		let tiles = vec
			.into_iter()
			.skip(4)
			.collect::<Vec<u8>>()
			.chunks_exact(16)
			.map(
				|chunk| {
					let mut x: [u8; 4] = Default::default();
					x.copy_from_slice(&chunk[0..4]);
					let mut y: [u8; 4] = Default::default();
					y.copy_from_slice(&chunk[4..8]);
					let mut raw_id: [u8; 8] = Default::default();
					raw_id.copy_from_slice(&chunk[8..16]);
					
					(TilePos(i32::from_le_bytes(x), i32::from_le_bytes(y)), WorldTile(RawId(u64::from_le_bytes(raw_id)), TileData(chunk[16])))
				}
			)
			.collect::<HashMap<_, _>>();
		
		Ok(
			Self {
				tiles,
			}
		)
	}
}
