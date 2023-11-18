use serde::{Deserialize, Serialize};
use bevy::prelude::Component;
use crate::{raw_id::RawId, identifier::Identifier, registry::tile::TileRegistry};

pub mod bundle;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldTile(pub RawId, pub TileData);

impl WorldTile {
	pub fn new_data(id: &Identifier, registry: &TileRegistry, data: TileData) -> Option<Self> {
		Some(Self(registry.get_raw_id(id)?, data))
	}
	
	pub fn new(id: &Identifier, registry: &TileRegistry) -> Option<Self> {
		WorldTile::new_data(id, registry, TileData(0))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Component)]
pub struct TileData(pub u8); // todo: advanced tile data
