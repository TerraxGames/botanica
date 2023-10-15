use bevy::prelude::*;

use crate::identifier::Identifier;
use crate::raw_id::RawId;
use crate::registry::Registry;
use crate::registry::tile::TileDef;

#[derive(Bundle)]
pub struct TileBundle {
	raw_id: RawId,
}

impl<'a> TileBundle {
	pub fn new(identifier: Identifier, registry: Registry<'a, TileDef<'a>>) -> Self {
		Self {
			raw_id: RawId(registry.get_id_hash(identifier)),
		}
	}

	pub fn new_raw(raw_id: RawId) -> Self {
		Self {
			raw_id
		}
	}
}
