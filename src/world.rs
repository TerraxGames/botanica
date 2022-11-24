use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::networking::protocol::ClientId;
use crate::TilePos;
use crate::raw_id::RawId;
use crate::registry::tile::TileData;

pub type WorldTile = (RawId, TileData);

pub struct Worlds(pub HashMap<String, WorldId>, pub HashMap<WorldId, GameWorld>);

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct GameWorld {
	id: WorldId,
	name: &'static str,
	tiles: HashMap<TilePos, WorldTile>,
	players: Vec<ClientId>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub u64);
