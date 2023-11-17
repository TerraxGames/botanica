use bevy::prelude::*;

use crate::asset::tile::TileDef;
use crate::identifier::Identifier;
use crate::raw_id::RawId;
use crate::registry::Registry;

#[derive(Bundle)]
pub struct TileBundle {
	raw_id: RawId,
}

impl<'a> TileBundle {
	pub fn new(identifier: &Identifier, registry: &Registry<TileDef>) -> Self {
		Self {
			raw_id: registry.get_raw_id(identifier),
		}
	}

	pub fn new_raw(raw_id: RawId) -> Self {
		Self {
			raw_id
		}
	}
}
