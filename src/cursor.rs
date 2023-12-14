use bevy::prelude::*;

use crate::TilePos;

#[derive(Component, Debug, Default, Clone)]
pub struct Cursor;

#[derive(Bundle, Default, Clone)]
pub struct CursorBundle {
	pub cursor: Cursor,
	pub position: TilePos,
	pub sprite_bundle: SpriteBundle,
}
