use std::collections::HashMap;
use std::fmt::Write;
use std::mem::size_of;
use bevy::reflect::List;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use serde::ser::SerializeStruct;
use crate::raw_id::RawId;
use crate::tile::TileData;
use crate::tile::WorldTile;
use crate::TilePos;

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
		vec.extend_from_slice(b"botanica_sav");
		vec.extend(tile_bytes);
		vec
	}
	
	pub fn deserialize(vec: Vec<u8>) -> Self {
		let tiles = vec
			.into_iter()
			.skip(12)
			.collect::<Vec<u8>>()
			.chunks_exact(size_of::<(TilePos, WorldTile)>())
			.map(
				|chunk| {
					let mut x: [u8; 8] = Default::default();
					x.copy_from_slice(&chunk[0..8]);
					let mut y: [u8; 8] = Default::default();
					y.copy_from_slice(&chunk[8..16]);
					let mut raw_id: [u8; 8] = Default::default();
					raw_id.copy_from_slice(&chunk[16..24]);
					
					(TilePos(u64::from_le_bytes(x), u64::from_le_bytes(y)), WorldTile(RawId(u64::from_le_bytes(raw_id)), TileData(chunk[24])))
				}
			)
			.collect::<HashMap<_, _>>();
		
		Self {
			tiles,
		}
	}
}
