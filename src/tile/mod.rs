use serde::{Serialize, Deserialize};

use crate::{raw_id::{RawId, tile::RawTileIds}, identifier::Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldTile(pub RawId, pub TileData);

impl WorldTile {
	pub fn new_data(id: &Identifier, raw_tile_ids: &RawTileIds, data: TileData) -> Option<Self> {
		Some(Self(raw_tile_ids.get_raw_id(id)?, data))
	}
	
	pub fn new(id: &Identifier, raw_tile_ids: &RawTileIds) -> Option<Self> {
		WorldTile::new_data(id, raw_tile_ids, TileData(0))
	}
	
	pub fn air() -> Self {
		Self(RawId(-1), TileData(0))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TileData(pub u8); // todo: advanced tile data
