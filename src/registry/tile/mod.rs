use bevy::prelude::*;
use crate::asset::tile::TileDef;
use crate::Registry;

pub mod settings;

#[derive(Deref, DerefMut, Resource)]
pub struct TileRegistry(Registry<TileDef>);

impl Default for TileRegistry {
	fn default() -> Self {
		Self(Registry::new())
	}
}
